//! Backtesting engine and reference strategies (Issue #25 / SPEC §47.2).

mod engine;
mod error;
mod metrics;
mod rsi_reversion;
mod sma_crossover;
mod strategy;

pub use engine::{run_backtest, strategy_from_params, verify_historical_symbol};
pub use error::BacktestError;
pub use rsi_reversion::RsiMeanReversionStrategy;
pub use sma_crossover::SmaCrossoverStrategy;
pub use strategy::{Bar, OrderIntent, OrderSide, Strategy, StrategyContext};
