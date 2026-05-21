//! Relative strength index with Wilder smoothing (Issue #21 / SPEC §46.1).

use crate::indicators::types::IndicatorSeries;

/// RSI with Wilder smoothing; first value at index `period` when `closes.len() > period`.
///
/// Returns an empty vector when `period == 0`. When insufficient history, all entries are `None`.
pub fn rsi(closes: &[f64], period: usize) -> IndicatorSeries {
    if period == 0 {
        return Vec::new();
    }
    let mut out = vec![None; closes.len()];
    if closes.len() <= period {
        return out;
    }

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..=period {
        let change = closes[i] - closes[i - 1];
        if change >= 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    out[period] = Some(rsi_from_averages(avg_gain, avg_loss));

    for i in (period + 1)..closes.len() {
        let change = closes[i] - closes[i - 1];
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
        out[i] = Some(rsi_from_averages(avg_gain, avg_loss));
    }
    out
}

fn rsi_from_averages(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        return if avg_gain == 0.0 { 50.0 } else { 100.0 };
    }
    if avg_gain == 0.0 {
        return 0.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_util::{assert_series_matches_fixture, fixture_closes};

    #[test]
    fn rsi_period_zero_returns_empty() {
        assert!(rsi(&[1.0, 2.0], 0).is_empty());
    }

    #[test]
    fn rsi_bounded_0_100() {
        let closes = fixture_closes();
        let s = rsi(&closes, 14);
        for v in s.into_iter().flatten() {
            assert!((0.0..=100.0).contains(&v), "rsi out of range: {v}");
        }
    }

    #[test]
    fn rsi_matches_fixture() {
        let closes = fixture_closes();
        let got = rsi(&closes, 14);
        assert_series_matches_fixture(&got, "indicators_rsi_14.json", 14);
    }
}
