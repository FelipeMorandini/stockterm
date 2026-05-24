//! Backtest fixture helpers (Issue #165 / SPEC §49.1).

use crate::models::backtest::{BacktestReport, BacktestSummary};

/// Golden summary fields from JSON fixtures (`tests/fixtures/backtest_*.json`).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GoldenSummaryExpect {
    pub trade_count: usize,
    pub total_pnl: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub win_rate_pct: f64,
    pub sharpe: f64,
    pub final_equity: f64,
}

/// §46.1 / §49.1 — returns true when `a` and `b` match within absolute or relative tolerance.
pub fn approx_eq(a: f64, b: f64) -> bool {
    const REL_EPS: f64 = 1e-6;
    const ABS_EPS: f64 = 1e-4;
    if !a.is_finite() || !b.is_finite() {
        return a == b;
    }
    let diff = (a - b).abs();
    diff <= ABS_EPS || diff <= REL_EPS * a.abs().max(b.abs())
}

/// Asserts `report.summary` and final equity match golden `expected` within §46.1 tolerance.
///
/// Panics with a field-specific message on mismatch (test-only).
pub fn assert_summary_matches(report: &BacktestReport, expected: &GoldenSummaryExpect) {
    assert_summary_matches_summary(&report.summary, expected);
    let final_eq = report.equity_curve.last().map(|(_, e)| *e).unwrap_or(0.0);
    assert!(
        approx_eq(final_eq, expected.final_equity),
        "final_equity: got {final_eq}, expected {}",
        expected.final_equity
    );
}

/// Asserts summary fields only (no equity-curve final point).
///
/// Panics with a field-specific message on mismatch (test-only).
pub fn assert_summary_matches_summary(actual: &BacktestSummary, expected: &GoldenSummaryExpect) {
    assert_eq!(
        actual.trade_count, expected.trade_count,
        "trade_count mismatch"
    );
    assert_field_approx(actual.total_pnl, expected.total_pnl, "total_pnl");
    assert_field_approx(
        actual.total_return_pct,
        expected.total_return_pct,
        "total_return_pct",
    );
    assert_field_approx(
        actual.max_drawdown_pct,
        expected.max_drawdown_pct,
        "max_drawdown_pct",
    );
    assert_field_approx(actual.win_rate_pct, expected.win_rate_pct, "win_rate_pct");
    assert_field_approx(actual.sharpe, expected.sharpe, "sharpe");
}

fn assert_field_approx(actual: f64, expected: f64, name: &str) {
    assert!(
        approx_eq(actual, expected),
        "{name}: got {actual}, expected {expected}"
    );
}
