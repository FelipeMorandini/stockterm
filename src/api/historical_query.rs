//! Parameters for [`MarketDataProvider::get_historical`](crate::api::provider::MarketDataProvider).

/// Provider-agnostic historical chart request (M4 / Issue #9).
#[derive(Debug, Clone, Copy)]
pub struct HistoricalQuery<'a> {
    pub from: &'a str,
    pub to: &'a str,
    /// Yahoo v8 `interval=` (e.g. `5m`, `1d`, `1wk`).
    pub bar_interval: &'a str,
    /// When set, Yahoo uses `range=` + `interval=` and ignores calendar `from`/`to`.
    pub yahoo_range: Option<&'static str>,
    pub polygon_multiplier: u32,
    pub polygon_timespan: &'a str,
}
