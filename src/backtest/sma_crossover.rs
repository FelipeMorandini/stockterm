//! SMA crossover strategy (Issue #25 / SPEC §47.2.3).

use crate::backtest::strategy::{Bar, OrderIntent, OrderSide, Strategy, StrategyContext};
use crate::indicators::sma;
use crate::indicators::types::IndicatorSeries;

/// Buy when fast SMA crosses above slow; sell when fast crosses below slow.
///
/// Indicator series are precomputed for the full close history at construction time.
pub struct SmaCrossoverStrategy {
    slow_period: usize,
    fast: IndicatorSeries,
    slow: IndicatorSeries,
}

impl SmaCrossoverStrategy {
    /// Builds indicator series for the full close history.
    pub fn new(closes: &[f64], fast_period: usize, slow_period: usize) -> Self {
        Self {
            slow_period,
            fast: sma(closes, fast_period),
            slow: sma(closes, slow_period),
        }
    }
}

impl Strategy for SmaCrossoverStrategy {
    fn on_bar(&mut self, bar: Bar<'_>, ctx: &StrategyContext<'_>) -> Vec<OrderIntent> {
        let i = bar.index;
        if i == 0 || i >= self.fast.len() {
            return vec![];
        }
        let warmup = self.slow_period.saturating_sub(1);
        if i < warmup {
            return vec![];
        }
        let (Some(f_prev), Some(f_curr), Some(s_prev), Some(s_curr)) = (
            self.fast.get(i - 1).copied().flatten(),
            self.fast.get(i).copied().flatten(),
            self.slow.get(i - 1).copied().flatten(),
            self.slow.get(i).copied().flatten(),
        ) else {
            return vec![];
        };

        if f_prev <= s_prev && f_curr > s_curr && ctx.shares <= 0.0 && ctx.cash > 0.0 {
            return vec![OrderIntent {
                side: OrderSide::Buy,
            }];
        }
        if f_prev >= s_prev && f_curr < s_curr && ctx.shares > 0.0 {
            return vec![OrderIntent {
                side: OrderSide::Sell,
            }];
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::strategy::StrategyContext;
    use crate::models::historical::HistoricalData;

    #[test]
    fn sma_crossover_emits_buy_on_golden_cross() {
        let closes: Vec<f64> = (0..120)
            .map(|i| {
                if i < 60 {
                    100.0 - i as f64 * 0.5
                } else {
                    70.0 + (i - 60) as f64 * 0.8
                }
            })
            .collect();
        let mut strat = SmaCrossoverStrategy::new(&closes, 10, 20);
        let mut bought = false;
        for i in 20..closes.len() {
            let ctx = StrategyContext {
                cash: 10_000.0,
                shares: if bought { 1.0 } else { 0.0 },
                closes: &closes,
                index: i,
            };
            let data = HistoricalData {
                o: closes[i],
                h: closes[i],
                l: closes[i],
                c: closes[i],
                v: 0.0,
                t: i as u64,
                vw: closes[i],
                n: None,
            };
            let intents = strat.on_bar(
                Bar {
                    index: i,
                    data: &data,
                },
                &ctx,
            );
            if intents.iter().any(|o| o.side == OrderSide::Buy) {
                bought = true;
            }
        }
        assert!(bought, "expected at least one buy signal");
    }
}
