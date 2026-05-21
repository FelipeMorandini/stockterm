//! Backtest configuration and report types (Issue #25 / SPEC §47.1).

use serde::{Deserialize, Serialize};

/// User-tunable simulation parameters (persisted in `~/.stockterm.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestConfig {
    #[serde(default = "default_initial_capital")]
    pub initial_capital: f64,
    #[serde(default)]
    pub commission_per_trade: f64,
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: f64,
}

fn default_initial_capital() -> f64 {
    10_000.0
}

fn default_slippage_bps() -> f64 {
    5.0
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            commission_per_trade: 0.0,
            slippage_bps: 5.0,
        }
    }
}

/// Built-in strategy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacktestStrategyKind {
    #[default]
    SmaCrossover,
    RsiMeanReversion,
}

/// Strategy parameters persisted with config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestStrategyParams {
    #[serde(default)]
    pub kind: BacktestStrategyKind,
    #[serde(default = "default_sma_fast")]
    pub sma_fast: usize,
    #[serde(default = "default_sma_slow")]
    pub sma_slow: usize,
    #[serde(default = "default_rsi_period")]
    pub rsi_period: usize,
    #[serde(default = "default_rsi_oversold")]
    pub rsi_oversold: f64,
    #[serde(default = "default_rsi_overbought")]
    pub rsi_overbought: f64,
}

fn default_sma_fast() -> usize {
    50
}

fn default_sma_slow() -> usize {
    200
}

fn default_rsi_period() -> usize {
    14
}

fn default_rsi_oversold() -> f64 {
    30.0
}

fn default_rsi_overbought() -> f64 {
    70.0
}

impl Default for BacktestStrategyParams {
    fn default() -> Self {
        Self {
            kind: BacktestStrategyKind::default(),
            sma_fast: default_sma_fast(),
            sma_slow: default_sma_slow(),
            rsi_period: default_rsi_period(),
            rsi_oversold: default_rsi_oversold(),
            rsi_overbought: default_rsi_overbought(),
        }
    }
}

/// One round-trip trade (v1 long-only).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeRecord {
    pub entry_ts: u64,
    pub exit_ts: u64,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub shares: f64,
    pub pnl: f64,
}

/// Aggregate metrics for a completed backtest run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub symbol: String,
    pub bar_count: usize,
    pub trade_count: usize,
    pub total_pnl: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub win_rate_pct: f64,
    pub sharpe: f64,
}

/// Full backtest output for UI and export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestReport {
    pub summary: BacktestSummary,
    pub trades: Vec<TradeRecord>,
    pub equity_curve: Vec<(u64, f64)>,
}

/// JSON export wrapper including config snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestExportBundle {
    pub report: BacktestReport,
    pub sim: BacktestConfig,
    pub strategy: BacktestStrategyParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_config_defaults_from_empty_json() {
        let cfg: BacktestConfig = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(cfg.initial_capital, 10_000.0);
        assert_eq!(cfg.commission_per_trade, 0.0);
        assert_eq!(cfg.slippage_bps, 5.0);
    }

    #[test]
    fn backtest_strategy_params_defaults_from_empty_json() {
        let p: BacktestStrategyParams = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(p.kind, BacktestStrategyKind::SmaCrossover);
        assert_eq!(p.sma_fast, 50);
        assert_eq!(p.sma_slow, 200);
    }
}
