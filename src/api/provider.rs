//! [`MarketDataProvider`] trait and factory for Yahoo vs Polygon.

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::MarketProviderKind;
use crate::config::Config;
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;

use crate::api::error::ProviderResult;
use crate::api::historical_query::HistoricalQuery;
use crate::api::polygon::PolygonProvider;
use crate::api::yahoo::YahooProvider;

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_quote(&self, symbol: &str, config: &Config) -> ProviderResult<TickerResponse>;

    async fn get_historical(
        &self,
        symbol: &str,
        query: &HistoricalQuery<'_>,
        config: &Config,
    ) -> ProviderResult<HistoricalResponse>;

    async fn search_symbols(&self, query: &str, config: &Config) -> ProviderResult<SymbolSearchResponse>;

    async fn get_news(&self, symbol: &str, config: &Config) -> ProviderResult<NewsResponse>;
}

/// Shared handle for spawned quote tasks (cheap `Arc` clone).
pub fn market_provider_for(kind: MarketProviderKind) -> Arc<dyn MarketDataProvider + Send + Sync> {
    match kind {
        MarketProviderKind::Yahoo => Arc::new(YahooProvider),
        MarketProviderKind::Polygon => Arc::new(PolygonProvider),
    }
}
