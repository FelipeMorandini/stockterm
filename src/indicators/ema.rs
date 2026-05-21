//! Exponential moving average (Issue #21 / SPEC §46.1).

use crate::indicators::types::IndicatorSeries;

/// Exponential moving average: \(\alpha = 2 / (period + 1)\), seeded with SMA at index `period - 1`.
///
/// Returns an empty vector when `period == 0`. When `closes.len() < period`, every entry is `None`.
pub fn ema(closes: &[f64], period: usize) -> IndicatorSeries {
    if period == 0 {
        return Vec::new();
    }
    let mut out = vec![None; closes.len()];
    if closes.len() < period {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = closes[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..closes.len() {
        prev = alpha * closes[i] + (1.0 - alpha) * prev;
        out[i] = Some(prev);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_util::{assert_series_matches_fixture, fixture_closes};

    #[test]
    fn ema_period_zero_returns_empty() {
        assert!(ema(&[1.0, 2.0], 0).is_empty());
    }

    #[test]
    fn ema_warmup_none_until_period() {
        let s = ema(&[1.0, 2.0, 3.0, 4.0, 5.0], 3);
        assert!(s[0].is_none() && s[1].is_none());
        assert!(s[2].is_some());
    }

    #[test]
    fn ema_matches_fixture() {
        let closes = fixture_closes();
        let got = ema(&closes, 20);
        assert_series_matches_fixture(&got, "indicators_ema_20.json", 20);
    }
}
