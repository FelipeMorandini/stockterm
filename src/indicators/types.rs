//! Shared types for technical indicator series (Issue #21 / SPEC §46.1).

/// One scalar per input bar; `None` until the indicator has enough history.
pub type IndicatorSeries = Vec<Option<f64>>;

/// MACD line, signal line, and histogram aligned to the input close series.
#[derive(Debug, Clone, PartialEq)]
pub struct MacdOutput {
    pub macd: IndicatorSeries,
    pub signal: IndicatorSeries,
    pub histogram: IndicatorSeries,
}

/// Default SMA/EMA overlay period (Charts tab).
pub const SMA_PERIOD: usize = 20;
/// Default EMA overlay period (Charts tab).
pub const EMA_PERIOD: usize = 20;
/// Default RSI period (Charts tab).
pub const RSI_PERIOD: usize = 14;
/// MACD fast EMA period.
pub const MACD_FAST: usize = 12;
/// MACD slow EMA period.
pub const MACD_SLOW: usize = 26;
/// MACD signal EMA period.
pub const MACD_SIGNAL: usize = 9;
