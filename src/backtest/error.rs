//! Backtest engine errors (Issue #25 / SPEC §47.2.2).

use thiserror::Error;

/// Errors from [`crate::backtest::engine::run_backtest`].
#[derive(Debug, Error, PartialEq)]
pub enum BacktestError {
    #[error("no bars to backtest")]
    EmptyBars,
    #[error("initial capital must be positive")]
    InvalidCapital,
    #[error("SMA fast period must be less than slow period")]
    InvalidSmaPeriods,
    #[error("RSI oversold must be below overbought")]
    InvalidRsiThresholds,
    #[error("RSI period must be positive")]
    InvalidRsiPeriod,
    #[error("historical data is for {hist_ticker}, active symbol is {symbol}")]
    SymbolMismatch { hist_ticker: String, symbol: String },
    #[error("backtest task panicked")]
    TaskPanicked,
}
