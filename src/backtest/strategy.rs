//! Strategy trait and order intents (Issue #25 / SPEC §47.2.1).

use crate::models::historical::HistoricalData;

/// One completed bar in chronological order (oldest → newest).
pub struct Bar<'a> {
    /// Index into the bar series (`0` = oldest).
    pub index: usize,
    /// OHLCV bar at this index.
    pub data: &'a HistoricalData,
}

/// Order side for v1 long-only engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    /// Open or add a long position.
    Buy,
    /// Close a long position.
    Sell,
}

/// Signal emitted by a strategy on a single bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderIntent {
    /// Buy or sell intent for the simulator.
    pub side: OrderSide,
}

/// Read-only portfolio + series context for signal generation.
pub struct StrategyContext<'a> {
    /// Cash balance before processing intents on this bar.
    pub cash: f64,
    /// Shares held before processing intents on this bar.
    pub shares: f64,
    /// Close prices for the full series (shared across bars).
    pub closes: &'a [f64],
    /// Current bar index (same as [`Bar::index`]).
    pub index: usize,
}

/// Rule-based strategy evaluated bar-by-bar.
///
/// Implementations must be [`Send`] because backtests run on a blocking worker thread.
pub trait Strategy: Send {
    /// Called once per bar. Return an empty vector when no action.
    fn on_bar(&mut self, bar: Bar<'_>, ctx: &StrategyContext<'_>) -> Vec<OrderIntent>;
}
