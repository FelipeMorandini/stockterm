//! Backtest performance metrics (Issue #25 / SPEC §47.2.4).

use crate::models::backtest::{BacktestSummary, TradeRecord};

/// Peak-to-trough drawdown as a percentage of peak equity (0 if no peak).
pub fn max_drawdown_pct(equity_curve: &[(u64, f64)]) -> f64 {
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    for &(_, eq) in equity_curve {
        if !eq.is_finite() {
            continue;
        }
        if eq > peak {
            peak = eq;
        }
        if peak > 0.0 {
            let dd = (peak - eq) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

/// Fraction of closed trades with positive PnL × 100.
pub fn win_rate_pct(trades: &[TradeRecord]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }
    let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
    wins as f64 / trades.len() as f64 * 100.0
}

/// Sharpe on per-bar equity returns; scales by `sqrt(252)` when bars look daily-ish.
///
/// Formula: `mean(r) / std(r) * sqrt(scale)` where `r[i] = (eq[i] - eq[i-1]) / eq[i-1]`.
/// When median bar spacing ≥ 20 hours, `scale = 252`; else `scale = 252 * bars / year_span`.
pub fn sharpe_ratio(equity_curve: &[(u64, f64)]) -> f64 {
    if equity_curve.len() < 3 {
        return 0.0;
    }
    let mut returns = Vec::with_capacity(equity_curve.len() - 1);
    for w in equity_curve.windows(2) {
        let prev = w[0].1;
        let curr = w[1].1;
        if prev > 0.0 && prev.is_finite() && curr.is_finite() {
            returns.push((curr - prev) / prev);
        }
    }
    if returns.len() < 2 {
        return 0.0;
    }
    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| {
            let d = r - mean;
            d * d
        })
        .sum::<f64>()
        / returns.len() as f64;
    let std = variance.sqrt();
    if std < 1e-12 {
        return 0.0;
    }
    let scale = sharpe_annualization_scale(equity_curve);
    mean / std * scale.sqrt()
}

fn sharpe_annualization_scale(equity_curve: &[(u64, f64)]) -> f64 {
    if equity_curve.len() < 2 {
        return 252.0;
    }
    let mut gaps: Vec<f64> = Vec::new();
    for w in equity_curve.windows(2) {
        let dt = w[1].0.saturating_sub(w[0].0) as f64 / 1000.0;
        if dt > 0.0 {
            gaps.push(dt);
        }
    }
    if gaps.is_empty() {
        return 252.0;
    }
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = gaps[gaps.len() / 2];
    // ≥ 20 hours between bars → treat as daily
    if median >= 20.0 * 3600.0 {
        252.0
    } else {
        let span_sec = equity_curve
            .last()
            .map(|l| l.0)
            .unwrap_or(0)
            .saturating_sub(equity_curve.first().map(|f| f.0).unwrap_or(0)) as f64
            / 1000.0;
        if span_sec <= 0.0 {
            252.0
        } else {
            let bars_per_year = equity_curve.len() as f64 * (365.25 * 86400.0) / span_sec;
            bars_per_year.max(1.0)
        }
    }
}

/// Build summary from equity curve and closed trades.
pub fn build_summary(
    symbol: &str,
    bar_count: usize,
    initial_capital: f64,
    trades: &[TradeRecord],
    equity_curve: &[(u64, f64)],
) -> BacktestSummary {
    let final_equity = equity_curve
        .last()
        .map(|(_, e)| *e)
        .unwrap_or(initial_capital);
    let total_pnl = final_equity - initial_capital;
    let total_return_pct = if initial_capital > 0.0 {
        total_pnl / initial_capital * 100.0
    } else {
        0.0
    };
    BacktestSummary {
        symbol: symbol.to_string(),
        bar_count,
        trade_count: trades.len(),
        total_pnl,
        total_return_pct,
        max_drawdown_pct: max_drawdown_pct(equity_curve),
        win_rate_pct: win_rate_pct(trades),
        sharpe: sharpe_ratio(equity_curve),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_drawdown_simple_peak_trough() {
        let curve = vec![
            (0, 100.0),
            (1, 120.0),
            (2, 90.0),
            (3, 100.0),
        ];
        let dd = max_drawdown_pct(&curve);
        // (120 - 90) / 120 = 25%
        assert!((dd - 25.0).abs() < 1e-6);
    }

    #[test]
    fn win_rate_all_wins() {
        let trades = vec![
            TradeRecord {
                entry_ts: 0,
                exit_ts: 1,
                side: "long".into(),
                entry_price: 10.0,
                exit_price: 12.0,
                shares: 1.0,
                pnl: 2.0,
            },
            TradeRecord {
                entry_ts: 2,
                exit_ts: 3,
                side: "long".into(),
                entry_price: 10.0,
                exit_price: 11.0,
                shares: 1.0,
                pnl: 1.0,
            },
        ];
        assert!((win_rate_pct(&trades) - 100.0).abs() < 1e-6);
    }
}
