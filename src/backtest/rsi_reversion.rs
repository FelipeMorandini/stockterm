//! RSI mean-reversion strategy (Issue #25 / SPEC §47.2.3).

use crate::backtest::strategy::{Bar, OrderIntent, OrderSide, Strategy, StrategyContext};
use crate::indicators::rsi;
use crate::indicators::types::IndicatorSeries;

/// Buy when RSI is below oversold; sell when RSI is above overbought.
///
/// RSI series are precomputed for the full close history at construction time.
pub struct RsiMeanReversionStrategy {
    oversold: f64,
    overbought: f64,
    rsi: IndicatorSeries,
    warmup: usize,
}

impl RsiMeanReversionStrategy {
    /// Builds RSI series for the full close history.
    pub fn new(closes: &[f64], period: usize, oversold: f64, overbought: f64) -> Self {
        Self {
            oversold,
            overbought,
            rsi: rsi(closes, period),
            warmup: period,
        }
    }
}

impl Strategy for RsiMeanReversionStrategy {
    fn on_bar(&mut self, bar: Bar<'_>, ctx: &StrategyContext<'_>) -> Vec<OrderIntent> {
        let i = bar.index;
        if i < self.warmup || i >= self.rsi.len() {
            return vec![];
        }
        let Some(r) = self.rsi[i] else {
            return vec![];
        };
        if r < self.oversold && ctx.shares <= 0.0 && ctx.cash > 0.0 {
            return vec![OrderIntent {
                side: OrderSide::Buy,
            }];
        }
        if r > self.overbought && ctx.shares > 0.0 {
            return vec![OrderIntent {
                side: OrderSide::Sell,
            }];
        }
        vec![]
    }
}
