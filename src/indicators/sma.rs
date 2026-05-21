//! Simple moving average (Issue #21 / SPEC §46.1).

use crate::indicators::types::IndicatorSeries;

/// Simple moving average over `closes` with window `period`.
///
/// Returns an empty vector when `period == 0`. When `closes.len() < period`, every
/// entry is `None`. Otherwise index `i` is `Some` once `i >= period - 1`.
pub fn sma(closes: &[f64], period: usize) -> IndicatorSeries {
    if period == 0 {
        return Vec::new();
    }
    let mut out = vec![None; closes.len()];
    if closes.len() < period {
        return out;
    }
    let mut sum = closes[..period].iter().sum::<f64>();
    out[period - 1] = Some(sum / period as f64);
    for i in period..closes.len() {
        sum += closes[i] - closes[i - period];
        out[i] = Some(sum / period as f64);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_util::{assert_series_matches_fixture, fixture_closes};

    #[test]
    fn sma_period_zero_returns_empty() {
        assert!(sma(&[1.0, 2.0], 0).is_empty());
    }

    #[test]
    fn sma_warmup_is_none_until_period() {
        let s = sma(&[1.0, 2.0, 3.0, 4.0, 5.0], 3);
        assert_eq!(s.len(), 5);
        assert!(s[0].is_none() && s[1].is_none());
        assert!((s[2].unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn sma_matches_fixture() {
        let closes = fixture_closes();
        let got = sma(&closes, 20);
        assert_series_matches_fixture(&got, "indicators_sma_20.json", 20);
    }
}
