//! Polygon.io [`MarketDataProvider`](crate::api::provider::MarketDataProvider) implementation.

use async_trait::async_trait;
use chrono::{Duration, Local};
use urlencoding::encode;

use crate::api::error::{map_reqwest, ProviderError, ProviderResult};
use crate::api::historical_query::HistoricalQuery;
use crate::api::http::shared_client;
use crate::api::provider::MarketDataProvider;
use crate::config::Config;
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;

const BASE_URL: &str = "https://api.polygon.io";

fn enc(s: &str) -> String {
    encode(s).into_owned()
}

fn polygon_key(config: &Config) -> ProviderResult<String> {
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
    let text = resp.text().await.map_err(map_reqwest)?;
    serde_json::from_str(&text).map_err(ProviderError::from)
}

pub struct PolygonProvider;

#[async_trait]
impl MarketDataProvider for PolygonProvider {
    async fn get_quote(&self, symbol: &str, config: &Config) -> ProviderResult<TickerResponse> {
        let key = polygon_key(config)?;
        let to = Local::now().format("%Y-%m-%d").to_string();
        let from = (Local::now() - Duration::days(30)).format("%Y-%m-%d").to_string();
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/1/day/{}/{}?adjusted=true&sort=desc&limit=120&apiKey={}",
            BASE_URL,
            enc(symbol),
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
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted=true&sort=asc&limit=50000&apiKey={}",
            BASE_URL,
            enc(symbol),
            query.polygon_multiplier,
            enc(query.polygon_timespan),
            enc(query.from),
            enc(query.to),
            enc(&key)
        );
        fetch_json(&url).await
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
            enc(symbol),
            enc(&key)
        );
        fetch_json(&url).await
    }
}
