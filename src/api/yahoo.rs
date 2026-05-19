//! Yahoo Finance (unofficial) [`MarketDataProvider`](crate::api::provider::MarketDataProvider).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use crate::api::concurrency::acquire_quote_permit;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use urlencoding::encode;

use crate::api::error::{ProviderError, ProviderResult};
use crate::api::historical_query::HistoricalQuery;
use crate::api::provider::MarketDataProvider;
use crate::api::retry::execute_get_text_with_retry;
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
    execute_get_text_with_retry(url).await
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

#[derive(Debug, Deserialize, Clone)]
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

fn empty_v7_ticker_response() -> TickerResponse {
    TickerResponse {
        ticker: String::new(),
        results: vec![],
        status: "OK".to_string(),
        error: None,
    }
}

/// Normalize a Yahoo **`symbol`** field for batch lookup (Issue #53 / SPEC §9.15.3).
fn normalize_v7_symbol_key(s: &str) -> String {
    s.trim().to_uppercase()
}

/// Maps one **`V7QuoteItem`** into [`TickerResponse`] (one synthetic bar). See table in [`v7_envelope_to_ticker`].
fn v7_item_to_ticker_response(q: &V7QuoteItem, requested: &str) -> ProviderResult<TickerResponse> {
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

/// Pick the v7 row for `requested`, or `None` when `items` is empty (Issue #91 / SPEC §34.5).
fn v7_select_item_for_symbol<'a>(
    items: &'a [V7QuoteItem],
    requested: &str,
) -> Option<&'a V7QuoteItem> {
    if items.is_empty() {
        return None;
    }
    if items.len() == 1 {
        return Some(&items[0]);
    }
    let key = normalize_v7_symbol_key(requested);
    items
        .iter()
        .find(|it| {
            it.symbol
                .as_deref()
                .is_some_and(|s| normalize_v7_symbol_key(s) == key)
        })
        .or_else(|| items.first())
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
        return Ok(empty_v7_ticker_response());
    };
    let Some(q) = v7_select_item_for_symbol(items, requested) else {
        return Ok(empty_v7_ticker_response());
    };

    v7_item_to_ticker_response(q, requested)
}

/// Last index wins if Yahoo returns duplicate **`symbol`** rows.
fn v7_rows_by_symbol_key(items: &[V7QuoteItem]) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for (i, it) in items.iter().enumerate() {
        if let Some(sym) = it.symbol.as_ref() {
            m.insert(normalize_v7_symbol_key(sym), i);
        }
    }
    m
}

/// Max length of the full **`v7/finance/quote`** request URL (Issue #53 / SPEC §9.15.5).
pub(crate) const YAHOO_V7_QUOTE_SYMBOLS_MAX_URL_BYTES: usize = 3000;

fn chunk_symbols_for_v7_quote_url_with_budget(symbols: &[String], max_url_bytes: usize) -> Vec<Vec<String>> {
    let base_len = format!("{}/v7/finance/quote?symbols=", QUERY1).len();
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    // Length of `sym1,sym2,...` (encoded) for symbols in `cur`.
    let mut cur_query_len: usize = 0;

    for s in symbols {
        let enc_len = encode(s.as_str()).len();
        let extra = if cur.is_empty() {
            enc_len
        } else {
            enc_len + 1
        };
        let new_total = cur_query_len + extra;
        if base_len + new_total <= max_url_bytes {
            cur.push(s.clone());
            cur_query_len = new_total;
            continue;
        }
        if !cur.is_empty() {
            chunks.push(std::mem::take(&mut cur));
            cur_query_len = 0;
        }
        // Retry `s` on empty `cur` (fall through to one-symbol oversized path or fit).
        if base_len + enc_len > max_url_bytes {
            chunks.push(vec![s.clone()]);
            continue;
        }
        cur.push(s.clone());
        cur_query_len = enc_len;
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    chunks
}

fn chunk_symbols_for_v7_quote_url(symbols: &[String]) -> Vec<Vec<String>> {
    chunk_symbols_for_v7_quote_url_with_budget(symbols, YAHOO_V7_QUOTE_SYMBOLS_MAX_URL_BYTES)
}

async fn yahoo_quote_v7_batch_chunk(chunk: &[String]) -> ProviderResult<V7QuoteEnvelope> {
    let joined = chunk
        .iter()
        .map(|s| encode(s.as_str()).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{}/v7/finance/quote?symbols={}", QUERY1, joined);
    let text = fetch_text(&url).await?;
    serde_json::from_str(&text).map_err(ProviderError::from)
}

/// Issue #53 / SPEC §9.15 — one primary **`v7`** GET per URL-size chunk, then per-symbol
/// [`yahoo_latest_quote`] for misses **or** when batched **`v7`** is unusable (HTTP errors
/// such as **401**, parse failures, or **`quoteResponse.error`**), so **`v8`** chart fallback
/// still applies the same way as pre-batch single-symbol quotes.
pub(crate) async fn yahoo_latest_quotes_for_symbols(
    symbols: &[String],
    max_concurrent_fallbacks: usize,
) -> (HashMap<String, TickerResponse>, Vec<(String, ProviderError)>) {
    let mut quotes = HashMap::new();
    let mut errors: Vec<(String, ProviderError)> = Vec::new();

    if symbols.is_empty() {
        return (quotes, errors);
    }

    let chunks = chunk_symbols_for_v7_quote_url(symbols);
    let mut pending_fallback: Vec<String> = Vec::new();

    for chunk in &chunks {
        match yahoo_quote_v7_batch_chunk(chunk).await {
            Ok(env) => {
                if env.quote_response.error.is_some() {
                    // Batched `v7` sometimes returns an in-JSON error (or a shape we treat as
                    // unusable). Per-symbol [`yahoo_latest_quote`] still tries `v7` then `v8`,
                    // matching pre–Issue #53 behavior when single-symbol `v7` is blocked.
                    pending_fallback.extend(chunk.iter().cloned());
                    continue;
                }

                let items = env.quote_response.result.unwrap_or_default();
                let index = v7_rows_by_symbol_key(&items);
                for sym in chunk {
                    let key = normalize_v7_symbol_key(sym);
                    match index.get(&key).copied() {
                        Some(idx) => match v7_item_to_ticker_response(&items[idx], sym) {
                            Ok(t) if !t.results.is_empty() => {
                                quotes.insert(sym.clone(), t);
                            }
                            Ok(_) | Err(_) => pending_fallback.push(sym.clone()),
                        },
                        None => pending_fallback.push(sym.clone()),
                    }
                }
            }
            Err(_) => {
                // HTTP failures (e.g. 401 on multi-symbol `v7`) or JSON parse errors: Yahoo
                // may reject batched `v7` while `v8` chart still works per symbol.
                pending_fallback.extend(chunk.iter().cloned());
            }
        }
    }

    let sem = Arc::new(Semaphore::new(max_concurrent_fallbacks.max(1)));
    let mut set = JoinSet::new();
    for sym in pending_fallback {
        let sem = sem.clone();
        set.spawn(async move {
            let _permit = match acquire_quote_permit(&sem, &sym, "yahoo").await {
                Ok(p) => p,
                Err(e) => return (sym, Err(e)),
            };
            let res = yahoo_latest_quote_at(&sym, QUERY1).await;
            (sym, res)
        });
    }

    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((sym, Ok(mut data))) => {
                if let Some(msg) = data.api_error_message() {
                    errors.push((sym, ProviderError::ApiMessage(msg)));
                    continue;
                }
                if data.ticker.is_empty() {
                    data.ticker = sym.clone();
                }
                quotes.insert(sym, data);
            }
            Ok((sym, Err(e))) => errors.push((sym, e)),
            Err(e) => errors.push((
                String::new(),
                ProviderError::Transport(format!("task join: {e}")),
            )),
        }
    }

    (quotes, errors)
}

fn v7_quote_url(query_base: &str, symbol: &str) -> String {
    let base = query_base.trim_end_matches('/');
    let enc_sym = encode(symbol);
    format!("{base}/v7/finance/quote?symbols={enc_sym}")
}

fn v8_chart_latest_url(query_base: &str, symbol: &str) -> String {
    let base = query_base.trim_end_matches('/');
    let enc_sym = encode(symbol);
    format!("{base}/v8/finance/chart/{enc_sym}?range=1d&interval=1d")
}

async fn yahoo_quote_v7_at(symbol: &str, query_base: &str) -> ProviderResult<TickerResponse> {
    let url = v7_quote_url(query_base, symbol);
    let text = fetch_text(&url).await?;
    let env: V7QuoteEnvelope = serde_json::from_str(&text)?;
    v7_envelope_to_ticker(&env, symbol)
}

/// Latest quote via **v8 chart** `range=1d` (fallback when v7 is empty or errors).
async fn yahoo_quote_at(symbol: &str, query_base: &str) -> ProviderResult<TickerResponse> {
    let url = v8_chart_latest_url(query_base, symbol);
    let text = fetch_text(&url).await?;
    let env: ChartEnvelope = serde_json::from_str(&text)?;
    chart_to_ticker(&env, symbol)
}

/// Enabled when `STOCKTERM_DEBUG_YAHOO_QUOTE` is exactly `1` (Issue #90 / SPEC §34.4).
fn yahoo_quote_fallback_debug_enabled() -> bool {
    std::env::var("STOCKTERM_DEBUG_YAHOO_QUOTE")
        .map(|s| s == "1")
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YahooV7FallbackReason {
    EmptyV7,
    V7Failed,
}

impl YahooV7FallbackReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::EmptyV7 => "empty_v7",
            Self::V7Failed => "v7_error",
        }
    }
}

/// Stderr diagnostic when v7 is unusable and v8 chart fallback runs (Issue #90 / SPEC §34.4).
fn maybe_log_yahoo_v7_fallback(symbol: &str, reason: YahooV7FallbackReason) {
    if !yahoo_quote_fallback_debug_enabled() {
        return;
    }
    eprintln!(
        "stockterm: yahoo quote {symbol}: v7 unusable ({}), using v8 chart",
        reason.as_str()
    );
}

/// Try **`v7/finance/quote`** first; on failure or empty body, use v8 chart ([`yahoo_quote_at`]).
async fn yahoo_latest_quote(symbol: &str) -> ProviderResult<TickerResponse> {
    yahoo_latest_quote_orchestrate(symbol, QUERY1, true).await
}

/// Same orchestration as [`yahoo_latest_quote`] with an injectable Yahoo **`query1`** base (Issue #89 / SPEC §32).
/// Does not emit §34 fallback stderr diagnostics (batch recovery and tests use this entry point).
pub(crate) async fn yahoo_latest_quote_at(
    symbol: &str,
    query_base: &str,
) -> ProviderResult<TickerResponse> {
    yahoo_latest_quote_orchestrate(symbol, query_base, false).await
}

async fn yahoo_latest_quote_orchestrate(
    symbol: &str,
    query_base: &str,
    log_v7_fallback: bool,
) -> ProviderResult<TickerResponse> {
    match yahoo_quote_v7_at(symbol, query_base).await {
        Ok(t) if !t.results.is_empty() => Ok(t),
        Ok(_) => {
            if log_v7_fallback {
                maybe_log_yahoo_v7_fallback(symbol, YahooV7FallbackReason::EmptyV7);
            }
            yahoo_quote_at(symbol, query_base).await
        }
        Err(_) => {
            if log_v7_fallback {
                maybe_log_yahoo_v7_fallback(symbol, YahooV7FallbackReason::V7Failed);
            }
            yahoo_quote_at(symbol, query_base).await
        }
    }
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
    let mut attempts = Vec::new();

    let search_result = yahoo_news_via_search(symbol).await;
    attempts.push(YahooNewsAttempt {
        source: YahooNewsSource::Search,
        outcome: yahoo_news_outcome_from_result(&search_result),
    });
    if let Ok(r) = search_result {
        maybe_log_yahoo_news_attempts(symbol, &attempts);
        return Ok(r);
    }

    let rss_result = yahoo_news_via_rss(symbol).await;
    attempts.push(YahooNewsAttempt {
        source: YahooNewsSource::Rss,
        outcome: yahoo_news_outcome_from_result(&rss_result),
    });
    if let Ok(r) = rss_result {
        maybe_log_yahoo_news_attempts(symbol, &attempts);
        return Ok(r);
    }

    let query2_result = yahoo_news_query2(symbol).await;
    attempts.push(YahooNewsAttempt {
        source: YahooNewsSource::Query2,
        outcome: yahoo_news_outcome_from_result(&query2_result),
    });
    maybe_log_yahoo_news_attempts(symbol, &attempts);
    query2_result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YahooNewsSource {
    Search,
    Rss,
    Query2,
}

impl YahooNewsSource {
    fn log_label(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Rss => "rss",
            Self::Query2 => "query2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum YahooNewsOutcome {
    OkItems { count: u32 },
    OkEmpty,
    ParseMismatch,
    ErrMessage(String),
}

struct YahooNewsAttempt {
    source: YahooNewsSource,
    outcome: YahooNewsOutcome,
}

/// Enabled when `STOCKTERM_DEBUG_YAHOO_NEWS` is exactly `1` (Issue #54 / SPEC §36.4.4).
fn yahoo_news_debug_enabled() -> bool {
    std::env::var("STOCKTERM_DEBUG_YAHOO_NEWS")
        .map(|s| s == "1")
        .unwrap_or(false)
}

fn yahoo_news_outcome_from_result(result: &ProviderResult<NewsResponse>) -> YahooNewsOutcome {
    match result {
        Ok(r) => {
            let count = r.results.len() as u32;
            if count > 0 {
                YahooNewsOutcome::OkItems { count }
            } else {
                YahooNewsOutcome::OkEmpty
            }
        }
        Err(ProviderError::ApiMessage(s)) if is_query2_news_shape_error(s) => {
            YahooNewsOutcome::ParseMismatch
        }
        Err(e) => YahooNewsOutcome::ErrMessage(truncate_utf8_debug_msg(&e.to_string())),
    }
}

const QUERY2_NEWS_SHAPE_ERR: &str =
    "Yahoo news (query2): response shape did not match known news JSON";

const YAHOO_NEWS_DEBUG_MSG_MAX_BYTES: usize = 120;

/// Returns true when `message` is the canonical query2 shape-mismatch error (Issue #54).
fn is_query2_news_shape_error(message: &str) -> bool {
    message == QUERY2_NEWS_SHAPE_ERR
}

/// Truncate debug stderr text at a UTF-8 scalar boundary (mirrors `alerts` notify body cap).
fn truncate_utf8_debug_msg(s: &str) -> String {
    truncate_utf8_to_max_bytes(s, YAHOO_NEWS_DEBUG_MSG_MAX_BYTES)
}

fn truncate_utf8_to_max_bytes(s: &str, max_bytes: usize) -> String {
    const ELLIPSIS: &str = "…";
    let el = ELLIPSIS.len();
    if s.len() <= max_bytes {
        return s.to_string();
    }
    if max_bytes < el {
        return String::new();
    }
    let prefix_max = max_bytes - el;
    let mut end = 0usize;
    for c in s.chars() {
        let cl = c.len_utf8();
        if end + cl > prefix_max {
            break;
        }
        end += cl;
    }
    format!("{}{}", &s[..end], ELLIPSIS)
}

fn maybe_log_yahoo_news_attempts(symbol: &str, attempts: &[YahooNewsAttempt]) {
    if !yahoo_news_debug_enabled() {
        return;
    }
    for attempt in attempts {
        let outcome = match &attempt.outcome {
            YahooNewsOutcome::OkItems { count } => format!("ok_items({count})"),
            YahooNewsOutcome::OkEmpty => "ok_empty".to_string(),
            YahooNewsOutcome::ParseMismatch => "parse_mismatch".to_string(),
            YahooNewsOutcome::ErrMessage(s) => format!("err({})", truncate_utf8_debug_msg(s)),
        };
        eprintln!(
            "stockterm: yahoo news {symbol}: {} {outcome}",
            attempt.source.log_label()
        );
    }
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
    yahoo_news_query2_from_text(&text, symbol)
}

/// Parses a `query2` news HTTP body (strict envelope, then lenient paths — Issue #54 / SPEC §36).
fn yahoo_news_query2_from_text(text: &str, symbol: &str) -> ProviderResult<NewsResponse> {
    let trimmed = text.trim_start();
    let value = match serde_json::from_str::<serde_json::Value>(text) {
        Ok(v) => v,
        Err(e) => {
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                return Err(ProviderError::ApiMessage(format!(
                    "Yahoo news (query2): JSON parse: {e}"
                )));
            }
            return Err(ProviderError::ApiMessage(
                "Yahoo news response was not valid JSON".to_string(),
            ));
        }
    };

    if let Ok(env) = serde_json::from_value::<NewsEnvelope>(value.clone()) {
        if let Some(stream) = env.data.and_then(|d| d.main).and_then(|m| m.stream) {
            return Ok(map_news_stream(stream, symbol));
        }
    }

    match query2_extract_stream_lenient_from_value(&value, symbol) {
        Query2LenientExtract::Stream(stream) => Ok(map_news_stream(stream, symbol)),
        Query2LenientExtract::Flat(items) => Ok(news_response_from_items(items)),
        Query2LenientExtract::Empty => Ok(empty_news_response()),
        Query2LenientExtract::NoMatch => Err(ProviderError::ApiMessage(
            QUERY2_NEWS_SHAPE_ERR.to_string(),
        )),
    }
}

enum Query2LenientExtract {
    Stream(Vec<NewsStreamItem>),
    Flat(Vec<NewsItem>),
    Empty,
    NoMatch,
}

fn empty_news_response() -> NewsResponse {
    NewsResponse {
        status: "OK".to_string(),
        count: 0,
        results: vec![],
    }
}

fn news_response_from_items(items: Vec<NewsItem>) -> NewsResponse {
    let count = items.len() as u32;
    NewsResponse {
        status: "OK".to_string(),
        count,
        results: items,
    }
}

/// Walk documented alternate `query2` JSON paths (SPEC §36.4.3).
fn query2_extract_stream_lenient_from_value(
    value: &serde_json::Value,
    symbol: &str,
) -> Query2LenientExtract {
    const STREAM_PATHS: &[&[&str]] = &[
        &["data", "main", "stream"],
        &["data", "stream"],
        &["main", "stream"],
        &["stream"],
    ];
    for path in STREAM_PATHS {
        if let Some(arr) = json_path(value, path).filter(|v| v.is_array()) {
            return stream_extract_from_array(arr, symbol);
        }
    }
    for key in ["items", "news"] {
        if let Some(arr) = value.get(key).filter(|v| v.is_array()) {
            if let Some(items) = map_query2_flat_news_array(arr, symbol) {
                if items.is_empty() {
                    return Query2LenientExtract::Empty;
                }
                return Query2LenientExtract::Flat(items);
            }
        }
    }
    Query2LenientExtract::NoMatch
}

fn json_path<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
    let mut cur = value;
    for key in path {
        cur = cur.get(*key)?;
    }
    Some(cur)
}

fn stream_extract_from_array(arr: &serde_json::Value, symbol: &str) -> Query2LenientExtract {
    if arr.as_array().is_some_and(|a| a.is_empty()) {
        return Query2LenientExtract::Empty;
    }
    if let Ok(stream) = serde_json::from_value::<Vec<NewsStreamItem>>(arr.clone()) {
        if stream.is_empty() {
            return Query2LenientExtract::Empty;
        }
        return Query2LenientExtract::Stream(stream);
    }
    if let Some(items) = map_query2_flat_news_array(arr, symbol) {
        if items.is_empty() {
            return Query2LenientExtract::Empty;
        }
        return Query2LenientExtract::Flat(items);
    }
    Query2LenientExtract::NoMatch
}

fn map_query2_flat_news_array(arr: &serde_json::Value, symbol: &str) -> Option<Vec<NewsItem>> {
    let rows = arr.as_array()?;
    let mut out = Vec::new();
    for row in rows {
        if let Ok(item) = serde_json::from_value::<NewsStreamItem>(row.clone()) {
            if let Some(mapped) = map_news_stream_item(item, symbol) {
                out.push(mapped);
            }
            continue;
        }
        if let Ok(flat) = serde_json::from_value::<Query2FlatNewsWire>(row.clone()) {
            if let Some(mapped) = map_query2_flat_wire(flat, symbol) {
                out.push(mapped);
            }
        }
    }
    if out.is_empty() && rows.is_empty() {
        return Some(vec![]);
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

fn map_news_stream_item(item: NewsStreamItem, symbol: &str) -> Option<NewsItem> {
    let content = item.content?;
    let title = content.title.filter(|t| !t.is_empty())?;
    let url = content
        .canonical_url
        .map(|c| c.url)
        .unwrap_or_default();
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
    Some(NewsItem {
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
    })
}

fn map_query2_flat_wire(wire: Query2FlatNewsWire, symbol: &str) -> Option<NewsItem> {
    let title = wire.title.filter(|t| !t.is_empty())?;
    let url = wire
        .link
        .or(wire.canonical_url.map(|c| c.url))
        .unwrap_or_default();
    let id = wire
        .id
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| format!("{:x}", md5_hash(&format!("{title}{url}"))));
    Some(NewsItem {
        id,
        publisher: Publisher {
            name: wire.publisher.unwrap_or_default(),
            homepage_url: String::new(),
            logo_url: String::new(),
            favicon_url: String::new(),
        },
        title,
        author: None,
        published_utc: wire.pub_date.unwrap_or_default(),
        article_url: url,
        tickers: vec![symbol.to_uppercase()],
        amp_url: None,
        image_url: None,
        description: wire.summary,
        keywords: vec![],
    })
}

fn map_news_stream(stream: Vec<NewsStreamItem>, symbol: &str) -> NewsResponse {
    let results: Vec<NewsItem> = stream
        .into_iter()
        .filter_map(|item| map_news_stream_item(item, symbol))
        .collect();
    news_response_from_items(results)
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

/// Flat `items` / `news` rows when `query2` drifts away from the stream envelope.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Query2FlatNewsWire {
    id: Option<String>,
    title: Option<String>,
    link: Option<String>,
    #[serde(rename = "canonicalUrl", alias = "canonical_url")]
    canonical_url: Option<CanonicalUrl>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    publisher: Option<String>,
    summary: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v7_batch_maps_rows_by_symbol_out_of_order() {
        let json = r#"{
            "quoteResponse": {
                "result": [
                    {
                        "symbol": "MSFT",
                        "regularMarketPrice": 400.0,
                        "regularMarketOpen": 399.0,
                        "regularMarketDayHigh": 401.0,
                        "regularMarketDayLow": 398.0,
                        "regularMarketVolume": 1000,
                        "regularMarketTime": 1700000001
                    },
                    {
                        "symbol": "AAPL",
                        "regularMarketPrice": 195.5,
                        "regularMarketOpen": 194.0,
                        "regularMarketDayHigh": 196.25,
                        "regularMarketDayLow": 193.5,
                        "regularMarketVolume": 52800000,
                        "regularMarketTime": 1700000000
                    }
                ],
                "error": null
            }
        }"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse v7");
        let items = env.quote_response.result.as_ref().expect("result");
        let index = v7_rows_by_symbol_key(items);
        let requested = ["AAPL", "MSFT"];
        for sym in requested {
            let idx = index
                .get(&normalize_v7_symbol_key(sym))
                .copied()
                .expect("row");
            let tr = v7_item_to_ticker_response(&items[idx], sym).expect("map");
            assert_eq!(tr.ticker.to_uppercase(), sym.to_uppercase());
            let bar = tr.latest_result().expect("bar");
            if sym == "AAPL" {
                assert!((bar.c - 195.5).abs() < 1e-9);
            } else {
                assert!((bar.c - 400.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn v7_chunk_splits_when_url_budget_small() {
        let syms: Vec<String> = (0..40).map(|i| format!("SYM{i}")).collect();
        let chunks = chunk_symbols_for_v7_quote_url_with_budget(&syms, 120);
        assert!(
            chunks.len() >= 2,
            "expected multiple chunks with tiny budget, got {}",
            chunks.len()
        );
        let flat: Vec<_> = chunks.iter().flatten().collect();
        assert_eq!(flat.len(), syms.len());
    }

    #[test]
    fn v7_envelope_picks_matching_symbol_among_many() {
        let json = r#"{
            "quoteResponse": {
                "result": [
                    {
                        "symbol": "MSFT",
                        "regularMarketPrice": 400.0,
                        "regularMarketOpen": 399.0,
                        "regularMarketDayHigh": 401.0,
                        "regularMarketDayLow": 398.0,
                        "regularMarketVolume": 1000,
                        "regularMarketTime": 1700000001
                    },
                    {
                        "symbol": "AAPL",
                        "regularMarketPrice": 195.5,
                        "regularMarketOpen": 194.0,
                        "regularMarketDayHigh": 196.25,
                        "regularMarketDayLow": 193.5,
                        "regularMarketVolume": 52800000,
                        "regularMarketTime": 1700000000
                    }
                ],
                "error": null
            }
        }"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse v7");
        let tr = v7_envelope_to_ticker(&env, "AAPL").expect("map");
        let bar = tr.latest_result().expect("bar");
        assert!((bar.c - 195.5).abs() < 1e-9);
    }

    #[test]
    fn v7_envelope_case_insensitive_symbol_match() {
        let json = r#"{
            "quoteResponse": {
                "result": [{
                    "symbol": "aapl",
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
        assert_eq!(tr.ticker, "aapl");
        let bar = tr.latest_result().expect("bar");
        assert!((bar.c - 195.5).abs() < 1e-9);
    }

    #[test]
    fn v7_envelope_no_symbol_match_falls_back_to_first() {
        let json = r#"{
            "quoteResponse": {
                "result": [
                    {
                        "symbol": "MSFT",
                        "regularMarketPrice": 400.0,
                        "regularMarketOpen": 399.0,
                        "regularMarketDayHigh": 401.0,
                        "regularMarketDayLow": 398.0,
                        "regularMarketVolume": 1000,
                        "regularMarketTime": 1700000001
                    },
                    {
                        "symbol": "GOOG",
                        "regularMarketPrice": 140.0,
                        "regularMarketOpen": 139.0,
                        "regularMarketDayHigh": 141.0,
                        "regularMarketDayLow": 138.0,
                        "regularMarketVolume": 2000,
                        "regularMarketTime": 1700000002
                    }
                ],
                "error": null
            }
        }"#;
        let env: V7QuoteEnvelope = serde_json::from_str(json).expect("parse v7");
        let tr = v7_envelope_to_ticker(&env, "AAPL").expect("map");
        let bar = tr.latest_result().expect("bar");
        assert!((bar.c - 400.0).abs() < 1e-9);
    }

    #[test]
    fn yahoo_quote_fallback_debug_enabled_respects_exact_one() {
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("STOCKTERM_DEBUG_YAHOO_QUOTE").ok();
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_QUOTE", "1");
        assert!(yahoo_quote_fallback_debug_enabled());
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_QUOTE", "0");
        assert!(!yahoo_quote_fallback_debug_enabled());
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_QUOTE", "true");
        assert!(!yahoo_quote_fallback_debug_enabled());
        match prev {
            Some(v) => std::env::set_var("STOCKTERM_DEBUG_YAHOO_QUOTE", v),
            None => std::env::remove_var("STOCKTERM_DEBUG_YAHOO_QUOTE"),
        }
    }

    #[test]
    fn maybe_log_yahoo_v7_fallback_no_panic_when_disabled() {
        maybe_log_yahoo_v7_fallback("AAPL", YahooV7FallbackReason::EmptyV7);
        maybe_log_yahoo_v7_fallback("MSFT", YahooV7FallbackReason::V7Failed);
    }

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
    fn yahoo_news_query2_maps_fixture_stream() {
        let json = include_str!("../../tests/fixtures/yahoo_news_query2_stream.json");
        let r = yahoo_news_query2_from_text(json, "AAPL").expect("parse stream fixture");
        assert!(r.count > 0);
        assert!(!r.results.is_empty());
        assert_eq!(r.results[0].title, "Test Headline");
    }

    #[test]
    fn yahoo_news_query2_drift_uses_lenient_path() {
        let json = include_str!("../../tests/fixtures/yahoo_news_query2_drift.json");
        let r = yahoo_news_query2_from_text(json, "MSFT").expect("parse drift fixture");
        assert!(r.count > 0);
        assert_eq!(r.results[0].title, "Drift Headline");
    }

    #[test]
    fn yahoo_news_query2_parse_mismatch_errors() {
        let err = yahoo_news_query2_from_text(r#"{"foo":1}"#, "AAPL").unwrap_err();
        match err {
            ProviderError::ApiMessage(s) => {
                assert!(s.contains("shape did not match"));
            }
            other => panic!("expected ApiMessage, got {:?}", other),
        }
    }

    #[test]
    fn yahoo_news_query2_empty_stream_is_ok_empty() {
        let json = r#"{"data":{"main":{"stream":[]}}}"#;
        let r = yahoo_news_query2_from_text(json, "AAPL").expect("empty stream");
        assert_eq!(r.count, 0);
        assert!(r.results.is_empty());
    }

    #[test]
    fn truncate_utf8_debug_msg_respects_scalar_boundary() {
        let s = "α".repeat(80);
        let out = truncate_utf8_debug_msg(&s);
        assert!(out.ends_with('…'));
        assert!(out.len() <= YAHOO_NEWS_DEBUG_MSG_MAX_BYTES);
    }

    #[test]
    fn is_query2_news_shape_error_matches_constant_only() {
        assert!(is_query2_news_shape_error(QUERY2_NEWS_SHAPE_ERR));
        assert!(!is_query2_news_shape_error(
            "Yahoo news (query2): response shape did not match known news JSON!"
        ));
    }

    #[test]
    fn yahoo_news_debug_enabled_respects_exact_one() {
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("STOCKTERM_DEBUG_YAHOO_NEWS").ok();
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_NEWS", "1");
        assert!(yahoo_news_debug_enabled());
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_NEWS", "0");
        assert!(!yahoo_news_debug_enabled());
        std::env::set_var("STOCKTERM_DEBUG_YAHOO_NEWS", "true");
        assert!(!yahoo_news_debug_enabled());
        match prev {
            Some(v) => std::env::set_var("STOCKTERM_DEBUG_YAHOO_NEWS", v),
            None => std::env::remove_var("STOCKTERM_DEBUG_YAHOO_NEWS"),
        }
    }

    #[test]
    fn maybe_log_yahoo_news_attempts_no_panic_when_disabled() {
        let attempts = vec![YahooNewsAttempt {
            source: YahooNewsSource::Query2,
            outcome: YahooNewsOutcome::ParseMismatch,
        }];
        maybe_log_yahoo_news_attempts("AAPL", &attempts);
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

    #[test]
    fn v7_quote_url_builds_expected_path() {
        let url = v7_quote_url("https://mock.test", "AAPL");
        assert_eq!(url, "https://mock.test/v7/finance/quote?symbols=AAPL");
    }

    #[test]
    fn v8_chart_latest_url_builds_expected_path() {
        let url = v8_chart_latest_url("https://mock.test/", "AAPL");
        assert_eq!(
            url,
            "https://mock.test/v8/finance/chart/AAPL?range=1d&interval=1d"
        );
    }
}

#[cfg(test)]
mod wiremock_quote_fallback_tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const SYMBOL: &str = "AAPL";
    const CHART_FIXTURE: &str = include_str!("../../tests/fixtures/yahoo_chart_aapl.json");
    const EXPECTED_CLOSE: f64 = 293.32;

    async fn mount_v8_chart_ok(srv: &MockServer) {
        Mock::given(method("GET"))
            .and(path(format!("/v8/finance/chart/{SYMBOL}")))
            .respond_with(ResponseTemplate::new(200).set_body_string(CHART_FIXTURE))
            .expect(1)
            .mount(srv)
            .await;
    }

    fn assert_chart_quote(tr: &TickerResponse) {
        let bar = tr.latest_result().expect("bar");
        assert!((bar.c - EXPECTED_CLOSE).abs() < 1e-9);
        assert_eq!(tr.ticker, "AAPL");
    }

    fn ensure_http_client() {
        crate::api::http::ensure_shared_client_for_tests();
    }

    #[tokio::test]
    async fn v7_malformed_json_falls_back_to_v8() {
        ensure_http_client();
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v7/finance/quote"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
            .expect(1)
            .mount(&srv)
            .await;
        mount_v8_chart_ok(&srv).await;

        let tr = yahoo_latest_quote_at(SYMBOL, &srv.uri())
            .await
            .expect("v8 fallback");
        assert_chart_quote(&tr);
    }

    #[tokio::test]
    async fn v7_empty_result_falls_back_to_v8() {
        ensure_http_client();
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v7/finance/quote"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"quoteResponse":{"result":[],"error":null}}"#,
            ))
            .expect(1)
            .mount(&srv)
            .await;
        mount_v8_chart_ok(&srv).await;

        let tr = yahoo_latest_quote_at(SYMBOL, &srv.uri())
            .await
            .expect("v8 fallback");
        assert_chart_quote(&tr);
    }

    #[tokio::test]
    async fn v7_api_error_envelope_falls_back_to_v8() {
        ensure_http_client();
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v7/finance/quote"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"quoteResponse":{"result":null,"error":{"description":"User is not logged in"}}}"#,
            ))
            .expect(1)
            .mount(&srv)
            .await;
        mount_v8_chart_ok(&srv).await;

        let tr = yahoo_latest_quote_at(SYMBOL, &srv.uri())
            .await
            .expect("v8 fallback");
        assert_chart_quote(&tr);
    }
}
