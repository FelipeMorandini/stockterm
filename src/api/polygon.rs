//! Polygon.io [`MarketDataProvider`](crate::api::provider::MarketDataProvider) implementation.

use async_trait::async_trait;
use chrono::{Duration, Local};
use urlencoding::encode;

use crate::api::error::{ProviderError, ProviderResult};
use crate::api::historical_query::HistoricalQuery;
use crate::api::polygon_pagination::{
    extend_historical_results, polygon_url_for_log, validate_polygon_next_url,
    POLYGON_HISTORICAL_MAX_PAGES,
};
use crate::api::provider::MarketDataProvider;
use crate::api::retry::execute_get_text_with_retry;
use crate::api::symbol::resolve_provider_symbol;
use crate::config::{Config, MarketProviderKind};
use crate::models::historical::{polygon_page_truncated, HistoricalResponse};
use crate::models::news::NewsResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;

const BASE_URL: &str = "https://api.polygon.io";

fn enc(s: &str) -> String {
    encode(s).into_owned()
}

fn polygon_wire_symbol(user_symbol: &str) -> String {
    resolve_provider_symbol(MarketProviderKind::Polygon, user_symbol)
}

/// API key for Polygon HTTP calls (shared with `polygon_options`).
pub(crate) fn polygon_key(config: &Config) -> ProviderResult<String> {
    let key = config.effective_api_key();
    if key.is_empty() {
        return Err(ProviderError::ApiMessage(
            "Polygon provider requires non-empty api_key in ~/.stockterm.json or STOCKTERM_API_KEY"
                .to_string(),
        ));
    }
    Ok(key.into_owned())
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> ProviderResult<T> {
    let text = execute_get_text_with_retry(url).await?;
    serde_json::from_str(&text).map_err(ProviderError::from)
}

fn map_historical_plan_error(msg: &str) -> ProviderError {
    if is_polygon_plan_message(msg) {
        ProviderError::ApiMessage(
            "Polygon plan does not include this aggregate window (try a shorter range or upgrade)"
                .into(),
        )
    } else {
        ProviderError::ApiMessage(msg.to_string())
    }
}

/// Follows Polygon `next_url` until exhausted or [`POLYGON_HISTORICAL_MAX_PAGES`] (Issue #176).
async fn fetch_polygon_historical_merged(initial_url: String) -> ProviderResult<HistoricalResponse> {
    let mut url = initial_url;
    let mut merged_results = Vec::new();
    let mut last = HistoricalResponse::default();
    let debug = std::env::var("STOCKTERM_DEBUG_POLYGON_HISTORICAL").as_deref() == Ok("1");
    let mut pages_fetched = 0usize;

    for page_idx in 0..POLYGON_HISTORICAL_MAX_PAGES {
        pages_fetched = page_idx + 1;
        if debug {
            tracing::info!(
                target: "stockterm::polygon",
                page = pages_fetched,
                url = polygon_url_for_log(&url),
                "polygon historical page fetch"
            );
        }

        let page: HistoricalResponse = fetch_json(&url).await?;
        if let Some(msg) = page.api_error_message() {
            return Err(map_historical_plan_error(&msg));
        }

        let next = page.next_url.clone();
        extend_historical_results(&mut merged_results, &page);
        last = page;

        if page_idx + 1 >= POLYGON_HISTORICAL_MAX_PAGES {
            break;
        }
        match next {
            Some(next) if !next.is_empty() => url = validate_polygon_next_url(&next)?,
            _ => break,
        }
    }

    let stopped_reason = if last
        .next_url
        .as_ref()
        .is_some_and(|s| !s.is_empty())
    {
        "page_cap"
    } else {
        "no_next_url"
    };

    tracing::info!(
        target: "stockterm::polygon",
        pages_fetched,
        results_len = merged_results.len(),
        results_count = last.results_count,
        stopped_reason,
        "polygon historical pagination complete"
    );

    let merged_len = merged_results.len() as u32;
    Ok(HistoricalResponse {
        results: merged_results,
        count: merged_len,
        results_count: last.results_count,
        next_url: last.next_url,
        status: last.status,
        ticker: last.ticker,
        request_id: last.request_id,
        error: None,
    })
}

pub struct PolygonProvider;

#[async_trait]
impl MarketDataProvider for PolygonProvider {
    /// Daily aggregates over a rolling calendar window; [`TickerResponse::latest_result`] (max `t`)
    /// is the **most recent bar** in the response — typically the last **US session** in range.
    async fn get_quote(&self, symbol: &str, config: &Config) -> ProviderResult<TickerResponse> {
        let key = polygon_key(config)?;
        let to = Local::now().format("%Y-%m-%d").to_string();
        let from = (Local::now() - Duration::days(30)).format("%Y-%m-%d").to_string();
        // `sort=desc` + small `limit`: latest session first, minimal payload (Issue #2 / SPEC §17.4).
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/1/day/{}/{}?adjusted=true&sort=desc&limit=5&apiKey={}",
            BASE_URL,
            enc(&polygon_wire_symbol(symbol)),
            enc(&from),
            enc(&to),
            enc(&key)
        );
        let ticker_data: TickerResponse = fetch_json(&url).await?;
        if let Some(msg) = ticker_data.api_error_message() {
            return Err(ProviderError::ApiMessage(msg));
        }
        Ok(ticker_data)
    }

    async fn get_historical(
        &self,
        symbol: &str,
        query: &HistoricalQuery<'_>,
        config: &Config,
    ) -> ProviderResult<HistoricalResponse> {
        let key = polygon_key(config)?;
        let limit = query.polygon_limit;
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted=true&sort=asc&limit={limit}&apiKey={}",
            BASE_URL,
            enc(&polygon_wire_symbol(symbol)),
            query.polygon_multiplier,
            enc(query.polygon_timespan),
            enc(query.from),
            enc(query.to),
            enc(&key)
        );
        let data = fetch_polygon_historical_merged(url).await?;
        if polygon_page_truncated(&data, limit) {
            tracing::warn!(
                target: "stockterm::polygon",
                limit,
                results_len = data.results.len(),
                results_count = data.results_count,
                has_next_url = data.next_url.is_some(),
                "polygon historical page truncated"
            );
        }
        Ok(data)
    }

    async fn search_symbols(&self, query: &str, config: &Config) -> ProviderResult<SymbolSearchResponse> {
        let key = polygon_key(config)?;
        let url = format!(
            "{}/v3/reference/tickers?search={}&active=true&apiKey={}",
            BASE_URL,
            enc(query),
            enc(&key)
        );
        fetch_json(&url).await
    }

    async fn get_news(&self, symbol: &str, config: &Config) -> ProviderResult<NewsResponse> {
        let key = polygon_key(config)?;
        let url = format!(
            "{}/v2/reference/news?ticker={}&apiKey={}",
            BASE_URL,
            enc(&polygon_wire_symbol(symbol)),
            enc(&key)
        );
        fetch_json(&url).await
    }

    async fn get_options_chain(
        &self,
        symbol: &str,
        expiration_ts: Option<u64>,
        config: &Config,
    ) -> ProviderResult<crate::models::options::OptionsChain> {
        Ok(
            crate::api::polygon_options::polygon_options_chain_with_slices(
                symbol,
                expiration_ts,
                config,
                None,
            )
            .await?
            .chain,
        )
    }
}

fn is_polygon_plan_message(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("not_authorized")
        || lower.contains("not authorized")
        || lower.contains("does not include")
        || lower.contains("subscription")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::historical::{polygon_page_truncated, HistoricalResponse};
    use crate::models::time_range::{polygon_historical_limit, TimeRange, POLYGON_AGG_LIMIT_CEILING};
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn polygon_historical_limit_ceiling() {
        for tr in [TimeRange::D1, TimeRange::W1, TimeRange::M1, TimeRange::Y1] {
            assert!(polygon_historical_limit(tr) <= POLYGON_AGG_LIMIT_CEILING);
        }
        assert!(polygon_historical_limit(TimeRange::D1) < 50_000);
        assert_eq!(polygon_historical_limit(TimeRange::D1), 500);
    }

    #[test]
    fn polygon_page_truncated_next_url() {
        let resp = HistoricalResponse {
            next_url: Some("https://api.polygon.io/next".into()),
            ..Default::default()
        };
        assert!(polygon_page_truncated(&resp, 500));
    }

    #[test]
    fn polygon_page_truncated_results_count() {
        let bar = crate::models::historical::HistoricalData {
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 0.0,
            t: 0,
            vw: 0.0,
            n: None,
        };
        let resp = HistoricalResponse {
            results_count: 10,
            results: vec![bar; 3],
            ..Default::default()
        };
        assert!(polygon_page_truncated(&resp, 500));
    }

    #[test]
    fn is_polygon_plan_message_detects_entitlement() {
        assert!(is_polygon_plan_message("NOT_AUTHORIZED"));
        assert!(is_polygon_plan_message("Your plan does not include this"));
        assert!(!is_polygon_plan_message("implementation plan rejected"));
    }

    #[test]
    fn polygon_page_truncated_full_page_not_flagged_without_next_url() {
        let bar = crate::models::historical::HistoricalData {
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 0.0,
            t: 0,
            vw: 0.0,
            n: None,
        };
        let resp = HistoricalResponse {
            results_count: 499,
            results: vec![bar; 499],
            ..Default::default()
        };
        assert!(!polygon_page_truncated(&resp, 500));
    }

    #[test]
    fn merge_historical_fixture_pages_concatenates_results() {
        let page1_text =
            std::fs::read_to_string(fixture_path("polygon_historical_page1.json")).unwrap();
        let page2_text =
            std::fs::read_to_string(fixture_path("polygon_historical_page2.json")).unwrap();
        let page1: HistoricalResponse = serde_json::from_str(&page1_text).unwrap();
        let page2: HistoricalResponse = serde_json::from_str(&page2_text).unwrap();
        let merged = crate::api::polygon_pagination::merge_historical_pages(&[page1, page2]);
        assert_eq!(merged.results.len(), 3);
        assert_eq!(merged.results_count, 3);
        assert!(merged.next_url.is_none());
        assert!(!polygon_page_truncated(&merged, 500));
    }

    #[test]
    fn fetch_stops_at_page_cap_keeps_next_url() {
        let bar = crate::models::historical::HistoricalData {
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 0.0,
            t: 0,
            vw: 0.0,
            n: None,
        };
        let page = HistoricalResponse {
            results: vec![bar],
            results_count: 100,
            next_url: Some("https://api.polygon.io/v2/next".into()),
            ..Default::default()
        };
        let pages = vec![page; POLYGON_HISTORICAL_MAX_PAGES];
        let merged = crate::api::polygon_pagination::merge_historical_pages(&pages);
        assert_eq!(merged.results.len(), POLYGON_HISTORICAL_MAX_PAGES);
        assert!(merged.next_url.is_some());
        assert!(polygon_page_truncated(&merged, 500));
    }
}
