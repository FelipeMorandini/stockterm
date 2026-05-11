//! Yahoo Finance (unofficial) [`MarketDataProvider`](crate::api::provider::MarketDataProvider).

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use urlencoding::encode;

use crate::api::error::{map_reqwest, ProviderError, ProviderResult};
use crate::api::historical_query::HistoricalQuery;
use crate::api::http::shared_client;
use crate::api::provider::MarketDataProvider;
use crate::config::Config;
use crate::models::historical::{HistoricalData, HistoricalResponse};
use crate::models::news::{NewsItem, NewsResponse, Publisher};
use crate::models::search::{SymbolResult, SymbolSearchResponse};
use crate::models::ticker::{TickerResponse, TickerResult};

const QUERY1: &str = "https://query1.finance.yahoo.com";
const QUERY2: &str = "https://query2.finance.yahoo.com";

/// Issue #73 / SPEC §11.12.3 — if the first W1 intraday response has no bars, retry with daily interval.
pub(crate) fn yahoo_w1_daily_fallback_interval(
    yahoo_range: Option<&str>,
    bar_interval: &str,
    first_result_count: usize,
) -> Option<&'static str> {
    if first_result_count != 0 {
        return None;
    }
    match (yahoo_range, bar_interval) {
        (Some("5d"), "30m") => Some("1d"),
        _ => None,
    }
}

pub struct YahooProvider;

#[async_trait]
impl MarketDataProvider for YahooProvider {
    async fn get_quote(&self, symbol: &str, config: &Config) -> ProviderResult<TickerResponse> {
        let _ = config;
        yahoo_latest_quote(symbol).await
    }

    async fn get_historical(
        &self,
        symbol: &str,
        query: &HistoricalQuery<'_>,
        config: &Config,
    ) -> ProviderResult<HistoricalResponse> {
        let _ = config;
        if let Some(range) = query.yahoo_range {
            let mut res = yahoo_historical_range(symbol, range, query.bar_interval).await?;
            // Issue #63 / SPEC §11.11.2 — W1 intraday empty → retry same window with daily bars.
            if let Some(iv) =
                yahoo_w1_daily_fallback_interval(Some(range), query.bar_interval, res.results.len())
            {
                res = yahoo_historical_range(symbol, "5d", iv).await?;
            }
            Ok(res)
        } else {
            yahoo_historical(symbol, query.from, query.to, query.bar_interval).await
        }
    }

    async fn search_symbols(&self, query: &str, config: &Config) -> ProviderResult<SymbolSearchResponse> {
        let _ = config;
        yahoo_search(query).await
    }

    async fn get_news(&self, symbol: &str, config: &Config) -> ProviderResult<NewsResponse> {
        let _ = config;
        yahoo_news(symbol).await
    }
}

async fn fetch_text(url: &str) -> ProviderResult<String> {
    let client = shared_client();
    let resp = client.get(url).send().await.map_err(map_reqwest)?;
    let status = resp.status();
    let url_owned = url.to_string();
    if !status.is_success() {
        return Err(ProviderError::Http {
            status: status.as_u16(),
            url: url_owned,
        });
    }
    resp.text().await.map_err(map_reqwest)
}

// --- v7/finance/quote (Issue #2 / SPEC §17) ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V7QuoteEnvelope {
    quote_response: V7QuoteResponseBody,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V7QuoteResponseBody {
    #[serde(default)]
    result: Option<Vec<V7QuoteItem>>,
    error: Option<V7QuoteWireError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V7QuoteWireError {
    description: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V7QuoteItem {
    symbol: Option<String>,
    regular_market_price: Option<f64>,
    regular_market_open: Option<f64>,
    regular_market_day_high: Option<f64>,
    regular_market_day_low: Option<f64>,
    #[serde(default)]
    regular_market_volume: Option<serde_json::Value>,
    regular_market_time: Option<i64>,
    regular_market_previous_close: Option<f64>,
}

fn v7_volume_as_f64(v: Option<&serde_json::Value>) -> f64 {
    let Some(j) = v else {
        return 0.0;
    };
    j.as_f64()
        .or_else(|| j.as_i64().map(|i| i as f64))
        .or_else(|| j.as_u64().map(|u| u as f64))
        .unwrap_or(0.0)
}

/// Maps Yahoo **`v7/finance/quote`** payload into [`TickerResponse`] (one synthetic bar).
///
/// | Yahoo field | `TickerResult` |
/// |-------------|----------------|
/// | `regularMarketOpen` | **`o`** (else `regularMarketPreviousClose`, else **`c`**) |
/// | `regularMarketDayHigh` | **`h`** (else **`c`**) |
/// | `regularMarketDayLow` | **`l`** (else **`c`**) |
/// | `regularMarketPrice` | **`c`** (required for a successful row) |
/// | `regularMarketVolume` | **`v`** |
/// | `regularMarketTime` (Unix **seconds**) | **`t`** = ms |
fn v7_envelope_to_ticker(env: &V7QuoteEnvelope, requested: &str) -> ProviderResult<TickerResponse> {
    if let Some(err) = &env.quote_response.error {
        let msg = err
            .description
            .clone()
            .or_else(|| err.code.clone())
            .unwrap_or_else(|| "Yahoo quote error".to_string());
        return Err(ProviderError::ApiMessage(msg));
    }
    let Some(items) = env.quote_response.result.as_ref() else {
        return Ok(TickerResponse {
            ticker: String::new(),
            results: vec![],
            status: "OK".to_string(),
            error: None,
        });
    };
    let Some(q) = items.first() else {
        return Ok(TickerResponse {
            ticker: String::new(),
            results: vec![],
            status: "OK".to_string(),
            error: None,
        });
    };

    let close = q.regular_market_price.ok_or_else(|| {
        ProviderError::ApiMessage(format!("No regularMarketPrice for {}", requested))
    })?;

    let open = q
        .regular_market_open
        .or(q.regular_market_previous_close)
        .unwrap_or(close);
    let high = q.regular_market_day_high.unwrap_or(close);
    let low = q.regular_market_day_low.unwrap_or(close);
    let vol = v7_volume_as_f64(q.regular_market_volume.as_ref());
    let t_sec = q
        .regular_market_time
        .unwrap_or_else(|| Utc::now().timestamp());
    let t_ms = (t_sec.max(0) as u64).saturating_mul(1000);

    let ticker_name = q
        .symbol
        .clone()
        .unwrap_or_else(|| requested.to_uppercase());

    Ok(TickerResponse {
        ticker: ticker_name,
        results: vec![TickerResult {
            o: open,
            h: high,
            l: low,
            c: close,
            v: vol,
            t: t_ms,
        }],
        status: "OK".to_string(),
        error: None,
    })
}

async fn yahoo_quote_v7(symbol: &str) -> ProviderResult<TickerResponse> {
    let enc_sym = encode(symbol);
    let url = format!("{}/v7/finance/quote?symbols={}", QUERY1, enc_sym);
    let text = fetch_text(&url).await?;
    let env: V7QuoteEnvelope = serde_json::from_str(&text)?;
    v7_envelope_to_ticker(&env, symbol)
}

/// Try **`v7/finance/quote`** first; on failure or empty body, use v8 chart ([`yahoo_quote`]).
async fn yahoo_latest_quote(symbol: &str) -> ProviderResult<TickerResponse> {
    match yahoo_quote_v7(symbol).await {
        Ok(t) if !t.results.is_empty() => Ok(t),
        Ok(_) | Err(_) => yahoo_quote(symbol).await,
    }
}

/// Latest quote via **v8 chart** `range=1d` (fallback when v7 is empty or errors).
async fn yahoo_quote(symbol: &str) -> ProviderResult<TickerResponse> {
    let enc_sym = encode(symbol);
    let url = format!(
        "{}/v8/finance/chart/{}?range=1d&interval=1d",
        QUERY1, enc_sym
    );
    let text = fetch_text(&url).await?;
    let env: ChartEnvelope = serde_json::from_str(&text)?;
    chart_to_ticker(&env, symbol)
}

fn chart_to_ticker(env: &ChartEnvelope, requested: &str) -> ProviderResult<TickerResponse> {
    if let Some(err) = &env.chart.error {
        let msg = err
            .description
            .clone()
            .or_else(|| err.code.clone())
            .unwrap_or_else(|| "Yahoo chart error".to_string());
        return Err(ProviderError::ApiMessage(msg));
    }
    let Some(results) = &env.chart.result else {
        return Err(ProviderError::ApiMessage(format!(
            "Unknown symbol: {}",
            requested
        )));
    };
    let Some(series) = results.first() else {
        return Err(ProviderError::ApiMessage(format!(
            "Unknown symbol: {}",
            requested
        )));
    };

    let meta = &series.meta;
    let close = meta
        .regular_market_price
        .or_else(|| last_close_from_bars(series))
        .ok_or_else(|| {
            ProviderError::ApiMessage(format!("No price data for {}", requested))
        })?;

    // Open: prefer session open, then chart previous close, then close.
    let open = meta
        .regular_market_open
        .or(meta.chart_previous_close)
        .unwrap_or(close);
    let high = meta.regular_market_day_high.unwrap_or(close);
    let low = meta.regular_market_day_low.unwrap_or(close);
    let vol = meta.regular_market_volume.map(|v| v as f64).unwrap_or(0.0);
    let t_sec = meta.regular_market_time.unwrap_or_else(|| Utc::now().timestamp());
    let t_ms = (t_sec.max(0) as u64).saturating_mul(1000);

    let ticker_name = meta
        .symbol
        .clone()
        .unwrap_or_else(|| requested.to_uppercase());

    Ok(TickerResponse {
        ticker: ticker_name,
        results: vec![TickerResult {
            o: open,
            h: high,
            l: low,
            c: close,
            v: vol,
            t: t_ms,
        }],
        status: "OK".to_string(),
        error: None,
    })
}

fn last_close_from_bars(series: &ChartSeries) -> Option<f64> {
    let ts = series.timestamp.as_ref()?;
    let quote = series.indicators.as_ref()?.quote.as_ref()?.first()?;
    let closes = quote.close.as_ref()?;
    let mut best: Option<(i64, f64)> = None;
    for (i, &t) in ts.iter().enumerate() {
        if let Some(Some(c)) = closes.get(i) {
            best = Some((t, *c));
        }
    }
    best.map(|(_, c)| c)
}

/// Yahoo v8 chart using `range=` + `interval=` (intraday and rolling windows).
async fn yahoo_historical_range(
    symbol: &str,
    range: &str,
    interval: &str,
) -> ProviderResult<HistoricalResponse> {
    let enc_sym = encode(symbol);
    let url = format!(
        "{}/v8/finance/chart/{}?range={}&interval={}",
        QUERY1, enc_sym, range, interval
    );
    let text = fetch_text(&url).await?;
    let env: ChartEnvelope = serde_json::from_str(&text)?;
    chart_to_historical(&env, symbol)
}

/// Calendar-bounded chart using `period1` / `period2` (Unix seconds) + `interval=`.
async fn yahoo_historical(
    symbol: &str,
    from_date: &str,
    to_date: &str,
    interval: &str,
) -> ProviderResult<HistoricalResponse> {
    let from = NaiveDate::parse_from_str(from_date, "%Y-%m-%d").map_err(|_| {
        ProviderError::ApiMessage(format!("Invalid from_date: {from_date}"))
    })?;
    let to = NaiveDate::parse_from_str(to_date, "%Y-%m-%d").map_err(|_| {
        ProviderError::ApiMessage(format!("Invalid to_date: {to_date}"))
    })?;
    let period1 = DateTime::<Utc>::from_naive_utc_and_offset(
        from.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
        Utc,
    )
    .timestamp();
    let period2 = DateTime::<Utc>::from_naive_utc_and_offset(
        to.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap()),
        Utc,
    )
    .timestamp();

    let enc_sym = encode(symbol);
    let url = format!(
        "{}/v8/finance/chart/{}?period1={}&period2={}&interval={}",
        QUERY1, enc_sym, period1, period2, interval
    );
    let text = fetch_text(&url).await?;
    let env: ChartEnvelope = serde_json::from_str(&text)?;
    chart_to_historical(&env, symbol)
}

fn chart_to_historical(env: &ChartEnvelope, requested: &str) -> ProviderResult<HistoricalResponse> {
    if let Some(err) = &env.chart.error {
        let msg = err
            .description
            .clone()
            .or_else(|| err.code.clone())
            .unwrap_or_else(|| "Yahoo chart error".to_string());
        return Err(ProviderError::ApiMessage(msg));
    }
    let Some(results) = &env.chart.result else {
        return Ok(HistoricalResponse {
            ticker: requested.to_uppercase(),
            results: vec![],
            status: "OK".to_string(),
            request_id: String::new(),
            count: 0,
        });
    };
    let Some(series) = results.first() else {
        return Ok(HistoricalResponse {
            ticker: requested.to_uppercase(),
            results: vec![],
            status: "OK".to_string(),
            request_id: String::new(),
            count: 0,
        });
    };

    let ticker = series
        .meta
        .symbol
        .clone()
        .unwrap_or_else(|| requested.to_uppercase());

    let timestamps = series.timestamp.clone().unwrap_or_default();
    let quote = series
        .indicators
        .as_ref()
        .and_then(|i| i.quote.as_ref())
        .and_then(|q| q.first());

    let mut out: Vec<HistoricalData> = Vec::new();
    if let Some(q) = quote {
        let opens = q.open.as_deref().unwrap_or(&[]);
        let highs = q.high.as_deref().unwrap_or(&[]);
        let lows = q.low.as_deref().unwrap_or(&[]);
        let closes = q.close.as_deref().unwrap_or(&[]);
        let vols = q.volume.as_deref().unwrap_or(&[]);

        for (i, &t_sec) in timestamps.iter().enumerate() {
            let c = match closes.get(i).and_then(|x| *x) {
                Some(x) => x,
                None => continue,
            };
            let o = opens.get(i).and_then(|x| *x).unwrap_or(c);
            let h = highs.get(i).and_then(|x| *x).unwrap_or(c);
            let l = lows.get(i).and_then(|x| *x).unwrap_or(c);
            let v = vols.get(i).and_then(|x| *x).unwrap_or(0.0);
            let vw = (o + h + l + c) / 4.0;
            let t_ms = (t_sec.max(0) as u64).saturating_mul(1000);
            out.push(HistoricalData {
                o,
                h,
                l,
                c,
                v,
                t: t_ms,
                vw,
                n: None,
            });
        }
    }

    out.sort_by_key(|b| b.t);
    let count = out.len() as u32;

    Ok(HistoricalResponse {
        ticker,
        results: out,
        status: "OK".to_string(),
        request_id: String::new(),
        count,
    })
}

async fn yahoo_search(query: &str) -> ProviderResult<SymbolSearchResponse> {
    let enc_q = encode(query);
    let url = format!(
        "{}/v1/finance/search?q={}&quotesCount=10",
        QUERY1, enc_q
    );
    let text = fetch_text(&url).await?;
    let env: SearchEnvelope = serde_json::from_str(&text)?;
    let quotes = env.quotes.unwrap_or_default();
    let results: Vec<SymbolResult> = quotes.into_iter().filter_map(map_search_quote).collect();
    let count = results.len() as u32;
    Ok(SymbolSearchResponse {
        status: "OK".to_string(),
        count,
        results,
    })
}

fn map_search_quote(q: SearchQuote) -> Option<SymbolResult> {
    let ticker = q.symbol?;
    let name = q
        .shortname
        .or(q.longname)
        .unwrap_or_else(|| ticker.clone());
    let market = q.exch_disp.or(q.exchange).unwrap_or_default();
    Some(SymbolResult {
        ticker,
        name,
        market: market.clone(),
        locale: "us".to_string(),
        primary_exchange: market,
        type_: q
            .quote_type
            .or(q.type_disp)
            .unwrap_or_else(|| "EQUITY".to_string()),
        active: true,
        currency_name: q.currency.unwrap_or_else(|| "USD".to_string()),
        cik: None,
        composite_figi: None,
        share_class_figi: None,
        last_updated_utc: String::new(),
    })
}

/// Yahoo `query2` **`/v2/finance/news`** often returns HTTP 500. Prefer **`query1` search**
/// (`newsCount`) and RSS, then keep query2 as a last resort.
async fn yahoo_news(symbol: &str) -> ProviderResult<NewsResponse> {
    if let Ok(r) = yahoo_news_via_search(symbol).await {
        return Ok(r);
    }
    if let Ok(r) = yahoo_news_via_rss(symbol).await {
        return Ok(r);
    }
    yahoo_news_query2(symbol).await
}

async fn yahoo_news_via_search(symbol: &str) -> ProviderResult<NewsResponse> {
    let enc = encode(symbol);
    let url = format!(
        "{}/v1/finance/search?q={}&newsCount=20&quotesCount=0",
        QUERY1, enc
    );
    let text = fetch_text(&url).await?;
    let env: SearchNewsEnvelope = serde_json::from_str(&text).map_err(|e| {
        ProviderError::ApiMessage(format!("Yahoo search/news JSON: {e}"))
    })?;
    let items = env.news.unwrap_or_default();
    let results: Vec<NewsItem> = items
        .into_iter()
        .filter_map(|w| map_search_news_wire(w, symbol))
        .collect();
    let count = results.len() as u32;
    Ok(NewsResponse {
        status: "OK".to_string(),
        count,
        results,
    })
}

fn map_search_news_wire(item: SearchNewsWire, symbol: &str) -> Option<NewsItem> {
    let title = item.title.filter(|t| !t.is_empty())?;
    let url = item.link.unwrap_or_default();
    let id = item
        .uuid
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| format!("{:x}", md5_hash(&format!("{title}{url}"))));
    let published_utc = item
        .provider_publish_time
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
        .map(|d| d.to_rfc3339())
        .unwrap_or_default();
    Some(NewsItem {
        id,
        publisher: Publisher {
            name: item.publisher.unwrap_or_default(),
            homepage_url: String::new(),
            logo_url: String::new(),
            favicon_url: String::new(),
        },
        title,
        author: None,
        published_utc,
        article_url: url,
        tickers: vec![symbol.to_uppercase()],
        amp_url: None,
        image_url: None,
        description: None,
        keywords: vec![],
    })
}

async fn yahoo_news_via_rss(symbol: &str) -> ProviderResult<NewsResponse> {
    let enc = encode(symbol);
    let url = format!(
        "https://feeds.finance.yahoo.com/rss/2.0/headline?s={}&region=US&lang=en-US",
        enc
    );
    let xml = fetch_text(&url).await?;
    let results = parse_yahoo_rss_items(&xml, symbol);
    if results.is_empty() {
        return Err(ProviderError::ApiMessage(
            "Yahoo RSS headline feed returned no items".to_string(),
        ));
    }
    let count = results.len() as u32;
    Ok(NewsResponse {
        status: "OK".to_string(),
        count,
        results,
    })
}

fn parse_yahoo_rss_items(xml: &str, symbol: &str) -> Vec<NewsItem> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<item>") {
        let after = &rest[start + 6..];
        let Some(end) = after.find("</item>") else {
            break;
        };
        let item_xml = &after[..end];
        rest = &after[end + 7..];
        let Some(title) = tag_inner_text(item_xml, "title") else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        let link = tag_inner_text(item_xml, "link").unwrap_or_default();
        let pub_date = tag_inner_text(item_xml, "pubDate").unwrap_or_default();
        let guid = tag_inner_text(item_xml, "guid").unwrap_or_default();
        let id = if guid.is_empty() {
            format!("{:x}", md5_hash(&format!("{title}{link}")))
        } else {
            guid
        };
        out.push(NewsItem {
            id,
            publisher: Publisher {
                name: "Yahoo Finance".to_string(),
                homepage_url: String::new(),
                logo_url: String::new(),
                favicon_url: String::new(),
            },
            title,
            author: None,
            published_utc: pub_date,
            article_url: link,
            tickers: vec![symbol.to_uppercase()],
            amp_url: None,
            image_url: None,
            description: None,
            keywords: vec![],
        });
    }
    out
}

fn tag_inner_text(block: &str, tag: &str) -> Option<String> {
    let needle = format!("<{tag}");
    let start = block.find(&needle)?;
    let from_tag = &block[start..];
    let gt = from_tag.find('>')?;
    let inner_start = start + gt + 1;
    let close = format!("</{tag}>");
    let rest = &block[inner_start..];
    let end = rest.find(&close)?;
    let raw = rest[..end].trim();
    Some(strip_cdata(raw))
}

fn strip_cdata(s: &str) -> String {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("<![CDATA[") {
        if let Some(x) = inner.strip_suffix("]]>") {
            return x.to_string();
        }
    }
    s.to_string()
}

async fn yahoo_news_query2(symbol: &str) -> ProviderResult<NewsResponse> {
    let enc_sym = encode(symbol);
    let url = format!("{}/v2/finance/news?symbols={}", QUERY2, enc_sym);
    let text = fetch_text(&url).await?;

    // Try structured stream format (common for query2).
    if let Ok(env) = serde_json::from_str::<NewsEnvelope>(&text) {
        if let Some(stream) = env.data.and_then(|d| d.main).and_then(|m| m.stream) {
            return Ok(map_news_stream(stream, symbol));
        }
    }

    // Fallback: empty list on parse failure only if body looks like JSON object without fatal shape.
    if text.trim_start().starts_with('{') {
        return Ok(NewsResponse {
            status: "OK".to_string(),
            count: 0,
            results: vec![],
        });
    }

    Err(ProviderError::ApiMessage(
        "Yahoo news response was not valid JSON".to_string(),
    ))
}

fn map_news_stream(stream: Vec<NewsStreamItem>, symbol: &str) -> NewsResponse {
    let mut results = Vec::new();
    for item in stream {
        let Some(content) = item.content else { continue };
        let title = content.title.unwrap_or_default();
        let url = content.canonical_url.map(|c| c.url).unwrap_or_default();
        let published = content
            .pub_date
            .or(content.provider_publish_time)
            .unwrap_or_default();
        let publisher_name = item
            .provider
            .as_ref()
            .and_then(|p| p.display_name.clone().or(p.name.clone()))
            .unwrap_or_default();

        let id = item
            .id
            .unwrap_or_else(|| format!("{:x}", md5_hash(&format!("{title}{url}"))));

        results.push(NewsItem {
            id,
            publisher: Publisher {
                name: publisher_name,
                homepage_url: String::new(),
                logo_url: String::new(),
                favicon_url: String::new(),
            },
            title,
            author: None,
            published_utc: published,
            article_url: url,
            tickers: vec![symbol.to_uppercase()],
            amp_url: None,
            image_url: None,
            description: content.summary,
            keywords: vec![],
        });
    }

    let count = results.len() as u32;
    NewsResponse {
        status: "OK".to_string(),
        count,
        results,
    }
}

fn md5_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// --- Yahoo wire JSON (serde) ------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartEnvelope {
    chart: ChartInner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartInner {
    result: Option<Vec<ChartSeries>>,
    error: Option<ChartApiError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartApiError {
    code: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartSeries {
    meta: ChartMeta,
    timestamp: Option<Vec<i64>>,
    indicators: Option<Indicators>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    symbol: Option<String>,
    #[serde(default)]
    regular_market_price: Option<f64>,
    #[serde(default)]
    regular_market_open: Option<f64>,
    #[serde(default)]
    regular_market_day_high: Option<f64>,
    #[serde(default)]
    regular_market_day_low: Option<f64>,
    #[serde(default)]
    regular_market_volume: Option<i64>,
    #[serde(default)]
    regular_market_time: Option<i64>,
    #[serde(default)]
    chart_previous_close: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Indicators {
    quote: Option<Vec<QuoteArrays>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteArrays {
    #[serde(default)]
    open: Option<Vec<Option<f64>>>,
    #[serde(default)]
    high: Option<Vec<Option<f64>>>,
    #[serde(default)]
    low: Option<Vec<Option<f64>>>,
    #[serde(default)]
    close: Option<Vec<Option<f64>>>,
    #[serde(default)]
    volume: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchEnvelope {
    quotes: Option<Vec<SearchQuote>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchNewsEnvelope {
    news: Option<Vec<SearchNewsWire>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchNewsWire {
    uuid: Option<String>,
    title: Option<String>,
    publisher: Option<String>,
    link: Option<String>,
    provider_publish_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuote {
    symbol: Option<String>,
    shortname: Option<String>,
    longname: Option<String>,
    exch_disp: Option<String>,
    exchange: Option<String>,
    quote_type: Option<String>,
    #[serde(rename = "typeDisp")]
    type_disp: Option<String>,
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsEnvelope {
    data: Option<NewsDataLayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsDataLayer {
    main: Option<NewsMainLayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsMainLayer {
    stream: Option<Vec<NewsStreamItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewsStreamItem {
    id: Option<String>,
    content: Option<StreamContent>,
    provider: Option<StreamProvider>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamContent {
    title: Option<String>,
    #[serde(rename = "canonicalUrl", alias = "canonical_url")]
    canonical_url: Option<CanonicalUrl>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    summary: Option<String>,
    #[serde(rename = "providerPublishTime")]
    provider_publish_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamProvider {
    name: Option<String>,
    display_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v7_envelope_maps_regular_market_fields() {
        let json = r#"{
            "quoteResponse": {
                "result": [{
                    "symbol": "AAPL",
                    "regularMarketPrice": 195.5,
                    "regularMarketOpen": 194.0,
                    "regularMarketDayHigh": 196.25,
                    "regularMarketDayLow": 193.5,
                    "regularMarketVolume": 52800000,
                    "regularMarketTime": 1700000000
                }],
                "error": null
            }
        }"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse v7");
        let tr = v7_envelope_to_ticker(&env, "AAPL").expect("map");
        assert_eq!(tr.ticker, "AAPL");
        let bar = tr.latest_result().expect("bar");
        assert!((bar.c - 195.5).abs() < 1e-9);
        assert!((bar.o - 194.0).abs() < 1e-9);
        assert!((bar.h - 196.25).abs() < 1e-9);
        assert!((bar.l - 193.5).abs() < 1e-9);
        assert!((bar.v - 52_800_000.0).abs() < 1.0);
        assert_eq!(bar.t, 1_700_000_000_000u64);
    }

    #[test]
    fn v7_envelope_empty_result_yields_no_bars_for_fallback() {
        let json = r#"{"quoteResponse":{"result":[],"error":null}}"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse");
        let tr = v7_envelope_to_ticker(&env, "ZZZZ").expect("ok envelope");
        assert!(tr.results.is_empty());
    }

    #[test]
    fn v7_envelope_api_error_returns_err() {
        let json = r#"{"quoteResponse":{"result":null,"error":{"description":"User is not logged in"}}}"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse");
        let e = v7_envelope_to_ticker(&env, "X").unwrap_err();
        match e {
            ProviderError::ApiMessage(s) => assert!(s.contains("logged in")),
            other => panic!("expected ApiMessage, got {:?}", other),
        }
    }

    #[test]
    fn v7_missing_price_returns_err() {
        let json = r#"{"quoteResponse":{"result":[{"symbol":"AAPL","regularMarketOpen":1.0}],"error":null}}"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse");
        assert!(v7_envelope_to_ticker(&env, "AAPL").is_err());
    }

    #[test]
    fn chart_to_ticker_fixture() {
        let json = include_str!("../../tests/fixtures/yahoo_chart_aapl.json");
        let env: ChartEnvelope = serde_json::from_str(json).expect("parse envelope");
        let tr = chart_to_ticker(&env, "AAPL").expect("map");
        assert_eq!(tr.status, "OK");
        assert!(!tr.results.is_empty());
        let bar = tr.latest_result().expect("bar");
        assert!(bar.c > 0.0);
        assert!(bar.t > 1_000_000_000_000); // ms
    }

    #[test]
    fn chart_to_historical_fixture() {
        let json = include_str!("../../tests/fixtures/yahoo_chart_aapl.json");
        let env: ChartEnvelope = serde_json::from_str(json).expect("parse envelope");
        let hist = chart_to_historical(&env, "AAPL").expect("hist");
        assert_eq!(hist.status, "OK");
        assert!(!hist.results.is_empty());
        assert!(hist.results[0].t > 1_000_000_000_000);
    }

    #[test]
    fn search_mapping_fixture() {
        let json = include_str!("../../tests/fixtures/yahoo_search_apple.json");
        let env: SearchEnvelope = serde_json::from_str(json).expect("parse");
        let quotes = env.quotes.unwrap_or_default();
        let mapped: Vec<SymbolResult> = quotes.into_iter().filter_map(map_search_quote).collect();
        assert!(!mapped.is_empty());
        assert!(!mapped[0].ticker.is_empty());
    }

    #[test]
    fn provider_error_display_smoke() {
        let e = ProviderError::Timeout;
        assert_eq!(e.to_string(), "Request timed out");
    }

    #[test]
    fn yahoo_search_news_maps_wire_row() {
        let json = r#"{"news":[{"uuid":"u1","title":"Hello","publisher":"Pub","link":"https://x","providerPublishTime":1700000000}]}"#;
        let env: SearchNewsEnvelope = serde_json::from_str(json).expect("parse");
        let w = env.news.expect("news").into_iter().next().expect("one");
        let n = map_search_news_wire(w, "AAPL").expect("map");
        assert_eq!(n.title, "Hello");
        assert_eq!(n.publisher.name, "Pub");
        assert_eq!(n.article_url, "https://x");
        assert!(n.published_utc.contains("2023"));
    }

    #[test]
    fn yahoo_rss_parses_minimal_item() {
        let xml = r#"<rss><channel><item><title>T1</title><link>https://a</link><pubDate>Mon, 1 Jan 2024 00:00:00 GMT</pubDate></item></channel></rss>"#;
        let v = parse_yahoo_rss_items(xml, "MSFT");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].title, "T1");
        assert_eq!(v[0].article_url, "https://a");
    }

    #[test]
    fn yahoo_w1_fallback_empty_intraday_requests_daily() {
        assert_eq!(
            yahoo_w1_daily_fallback_interval(Some("5d"), "30m", 0),
            Some("1d")
        );
    }

    #[test]
    fn yahoo_w1_fallback_skips_when_intraday_has_bars() {
        assert_eq!(yahoo_w1_daily_fallback_interval(Some("5d"), "30m", 3), None);
    }

    #[test]
    fn yahoo_w1_fallback_skips_wrong_range_or_interval() {
        assert_eq!(yahoo_w1_daily_fallback_interval(Some("1mo"), "30m", 0), None);
        assert_eq!(yahoo_w1_daily_fallback_interval(Some("5d"), "1d", 0), None);
        assert_eq!(yahoo_w1_daily_fallback_interval(None, "30m", 0), None);
    }
}
