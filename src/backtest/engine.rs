//! In-process backtest simulator (Issue #25 / SPEC §47.2.2).

use crate::models::historical::HistoricalResponse;

use crate::backtest::error::BacktestError;
use crate::backtest::metrics::build_summary;
use crate::backtest::strategy::{Bar, OrderSide, Strategy, StrategyContext};
use crate::models::backtest::{
    BacktestConfig, BacktestReport, BacktestStrategyKind, BacktestStrategyParams, TradeRecord,
};
use crate::models::historical::HistoricalData;

use super::rsi_reversion::RsiMeanReversionStrategy;
use super::sma_crossover::SmaCrossoverStrategy;

struct OpenPosition {
    entry_ts: u64,
    entry_price: f64,
    shares: f64,
}

/// Builds a boxed strategy from persisted params.
///
/// Validates SMA/RSI parameter ranges before constructing the strategy.
pub fn strategy_from_params(
    params: &BacktestStrategyParams,
    closes: &[f64],
) -> Result<Box<dyn Strategy>, BacktestError> {
    match params.kind {
        BacktestStrategyKind::SmaCrossover => {
            if params.sma_fast >= params.sma_slow || params.sma_fast == 0 {
                return Err(BacktestError::InvalidSmaPeriods);
            }
            Ok(Box::new(SmaCrossoverStrategy::new(
                closes,
                params.sma_fast,
                params.sma_slow,
            )))
        }
        BacktestStrategyKind::RsiMeanReversion => {
            if params.rsi_period == 0 {
                return Err(BacktestError::InvalidRsiPeriod);
            }
            if params.rsi_oversold >= params.rsi_overbought {
                return Err(BacktestError::InvalidRsiThresholds);
            }
            Ok(Box::new(RsiMeanReversionStrategy::new(
                closes,
                params.rsi_period,
                params.rsi_oversold,
                params.rsi_overbought,
            )))
        }
    }
}

fn fill_price(close: f64, side: OrderSide, slippage_bps: f64) -> f64 {
    let adj = slippage_bps / 10_000.0;
    match side {
        OrderSide::Buy => close * (1.0 + adj),
        OrderSide::Sell => close * (1.0 - adj),
    }
}

/// Returns `Ok(())` when `hist.ticker` is empty or matches `symbol` (case-insensitive).
pub fn verify_historical_symbol(
    hist: &HistoricalResponse,
    symbol: &str,
) -> Result<(), BacktestError> {
    let t = hist.ticker.trim();
    if t.is_empty() || t.eq_ignore_ascii_case(symbol) {
        return Ok(());
    }
    Err(BacktestError::SymbolMismatch {
        hist_ticker: t.to_string(),
        symbol: symbol.to_string(),
    })
}

/// Runs a backtest over `bars` with the given config and strategy.
///
/// Returns [`BacktestError`] for empty bars, invalid config/strategy params, or non-finite inputs
/// skipped at fill time. Does not perform HTTP I/O.
pub fn run_backtest(
    symbol: &str,
    bars: &[HistoricalData],
    sim: &BacktestConfig,
    params: &BacktestStrategyParams,
) -> Result<BacktestReport, BacktestError> {
    if bars.is_empty() {
        return Err(BacktestError::EmptyBars);
    }
    if sim.initial_capital <= 0.0 || !sim.initial_capital.is_finite() {
        return Err(BacktestError::InvalidCapital);
    }

    let closes: Vec<f64> = bars.iter().map(|b| b.c).collect();
    let mut strategy = strategy_from_params(params, &closes)?;

    let mut cash = sim.initial_capital;
    let mut shares = 0.0_f64;
    let mut open: Option<OpenPosition> = None;
    let mut trades: Vec<TradeRecord> = Vec::new();
    let mut equity_curve: Vec<(u64, f64)> = Vec::with_capacity(bars.len());

    for (i, bar) in bars.iter().enumerate() {
        let ctx = StrategyContext {
            cash,
            shares,
            closes: &closes,
            index: i,
        };
        let intents = strategy.on_bar(
            Bar {
                index: i,
                data: bar,
            },
            &ctx,
        );

        for intent in intents {
            match intent.side {
                OrderSide::Buy if shares <= 0.0 && cash > 0.0 => {
                    let price = fill_price(bar.c, OrderSide::Buy, sim.slippage_bps);
                    if price <= 0.0 || !price.is_finite() {
                        continue;
                    }
                    let spendable = (cash - sim.commission_per_trade).max(0.0);
                    let qty = spendable / price;
                    if qty <= 0.0 {
                        continue;
                    }
                    cash = 0.0;
                    shares = qty;
                    open = Some(OpenPosition {
                        entry_ts: bar.t,
                        entry_price: price,
                        shares: qty,
                    });
                }
                OrderSide::Sell if shares > 0.0 => {
                    let price = fill_price(bar.c, OrderSide::Sell, sim.slippage_bps);
                    let proceeds = shares * price - sim.commission_per_trade;
                    if let Some(pos) = open.take() {
                        let pnl =
                            proceeds - pos.entry_price * pos.shares - sim.commission_per_trade;
                        trades.push(TradeRecord {
                            entry_ts: pos.entry_ts,
                            exit_ts: bar.t,
                            side: "long".into(),
                            entry_price: pos.entry_price,
                            exit_price: price,
                            shares: pos.shares,
                            pnl,
                        });
                    }
                    cash = proceeds.max(0.0);
                    shares = 0.0;
                }
                _ => {}
            }
        }

        let equity = cash + shares * bar.c;
        equity_curve.push((bar.t, equity));
    }

    // Force flat on last bar
    if shares > 0.0 {
        let last = bars.last().expect("non-empty");
        let price = fill_price(last.c, OrderSide::Sell, sim.slippage_bps);
        let proceeds = shares * price - sim.commission_per_trade;
        if let Some(pos) = open.take() {
            let pnl = proceeds - pos.entry_price * pos.shares - sim.commission_per_trade;
            trades.push(TradeRecord {
                entry_ts: pos.entry_ts,
                exit_ts: last.t,
                side: "long".into(),
                entry_price: pos.entry_price,
                exit_price: price,
                shares: pos.shares,
                pnl,
            });
        }
        cash = proceeds.max(0.0);
        if let Some(last_point) = equity_curve.last_mut() {
            last_point.1 = cash;
        }
    }

    let summary = build_summary(
        symbol,
        bars.len(),
        sim.initial_capital,
        &trades,
        &equity_curve,
    );

    Ok(BacktestReport {
        summary,
        trades,
        equity_curve,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::backtest::{BacktestStrategyKind, BacktestStrategyParams};
    use crate::models::historical::HistoricalResponse;

    fn synthetic_bars(n: usize) -> Vec<HistoricalData> {
        (0..n)
            .map(|i| {
                let c = if i < n / 2 {
                    100.0 - i as f64 * 0.2
                } else {
                    80.0 + (i - n / 2) as f64 * 0.5
                };
                HistoricalData {
                    o: c,
                    h: c,
                    l: c,
                    c,
                    v: 1.0,
                    t: (i as u64 + 1) * 86_400_000,
                    vw: c,
                    n: None,
                }
            })
            .collect()
    }

    #[test]
    fn run_backtest_empty_bars_errors() {
        let sim = BacktestConfig::default();
        let params = BacktestStrategyParams::default();
        let err = run_backtest("TEST", &[], &sim, &params).unwrap_err();
        assert_eq!(err, BacktestError::EmptyBars);
    }

    #[test]
    fn verify_historical_symbol_mismatch() {
        let hist = HistoricalResponse {
            ticker: "MSFT".into(),
            results: vec![],
            status: String::new(),
            request_id: String::new(),
            count: 0,
            ..Default::default()
        };
        let err = verify_historical_symbol(&hist, "AAPL").unwrap_err();
        assert!(matches!(err, BacktestError::SymbolMismatch { .. }));
    }

    #[test]
    fn strategy_from_params_rejects_rsi_period_zero() {
        let closes = vec![100.0; 30];
        let params = BacktestStrategyParams {
            kind: BacktestStrategyKind::RsiMeanReversion,
            rsi_period: 0,
            ..Default::default()
        };
        assert!(matches!(
            strategy_from_params(&params, &closes),
            Err(BacktestError::InvalidRsiPeriod)
        ));
    }

    #[test]
    fn run_backtest_invalid_capital_errors() {
        let bars = synthetic_bars(50);
        let sim = BacktestConfig {
            initial_capital: 0.0,
            ..Default::default()
        };
        let params = BacktestStrategyParams::default();
        let err = run_backtest("TEST", &bars, &sim, &params).unwrap_err();
        assert_eq!(err, BacktestError::InvalidCapital);
    }

    #[test]
    fn run_backtest_sma_produces_report() {
        let bars = synthetic_bars(120);
        let sim = BacktestConfig::default();
        let params = BacktestStrategyParams {
            sma_fast: 10,
            sma_slow: 20,
            ..Default::default()
        };
        let report = run_backtest("TEST", &bars, &sim, &params).expect("ok");
        assert_eq!(report.summary.bar_count, 120);
        assert_eq!(report.equity_curve.len(), 120);
    }

    fn fixture_bars_from_closes(closes: &[f64]) -> Vec<HistoricalData> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| HistoricalData {
                o: c,
                h: c,
                l: c,
                c,
                v: 1.0,
                t: (i as u64 + 1) * 86_400_000,
                vw: c,
                n: None,
            })
            .collect()
    }

    #[test]
    fn run_backtest_fixture_sma_golden_metrics() {
        use crate::backtest::test_util::{assert_summary_matches, GoldenSummaryExpect};
        use serde::Deserialize;
        use std::fs;
        use std::path::PathBuf;

        #[derive(Deserialize)]
        struct Fixture {
            closes: Vec<f64>,
            sma_fast: usize,
            sma_slow: usize,
            initial_capital: f64,
            commission_per_trade: f64,
            slippage_bps: f64,
            expected: GoldenSummaryExpect,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/backtest_sma_crossover_50_200.json");
        let raw = fs::read_to_string(&path).expect("fixture");
        let fixture: Fixture = serde_json::from_str(&raw).expect("parse");

        let bars = fixture_bars_from_closes(&fixture.closes);
        let sim = BacktestConfig {
            initial_capital: fixture.initial_capital,
            commission_per_trade: fixture.commission_per_trade,
            slippage_bps: fixture.slippage_bps,
        };
        let params = BacktestStrategyParams {
            sma_fast: fixture.sma_fast,
            sma_slow: fixture.sma_slow,
            ..Default::default()
        };

        let report = run_backtest("FIX", &bars, &sim, &params).expect("ok");
        assert_summary_matches(&report, &fixture.expected);
    }

    #[test]
    fn backtest_fixture_deserializes() {
        use crate::backtest::test_util::GoldenSummaryExpect;
        use serde::Deserialize;
        use std::fs;
        use std::path::PathBuf;

        #[derive(Deserialize)]
        struct Fixture {
            closes: Vec<f64>,
            expected: GoldenSummaryExpect,
        }

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/backtest_sma_crossover_50_200.json");
        let raw = fs::read_to_string(&path).expect("fixture");
        let fixture: Fixture = serde_json::from_str(&raw).expect("parse");
        assert!(!fixture.closes.is_empty());
        assert!(fixture.expected.trade_count > 0);
    }
}
