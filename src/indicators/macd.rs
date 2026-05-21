//! MACD (moving average convergence divergence) (Issue #21 / SPEC §46.1).

use crate::indicators::ema::ema;
use crate::indicators::types::{IndicatorSeries, MacdOutput};

/// MACD line (fast EMA − slow EMA), signal EMA of the MACD line, and histogram (macd − signal).
pub fn macd(closes: &[f64], fast: usize, slow: usize, signal: usize) -> MacdOutput {
    let n = closes.len();
    if fast == 0 || slow == 0 || signal == 0 || n == 0 {
        return MacdOutput {
            macd: Vec::new(),
            signal: Vec::new(),
            histogram: Vec::new(),
        };
    }

    let fast_ema = ema(closes, fast);
    let slow_ema = ema(closes, slow);
    let mut macd_line = vec![None; n];
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_ema[i], slow_ema[i]) {
            macd_line[i] = Some(f - s);
        }
    }

    let signal_line = ema_optional(&macd_line, signal);
    let mut histogram = vec![None; n];
    for i in 0..n {
        if let (Some(m), Some(s)) = (macd_line[i], signal_line[i]) {
            histogram[i] = Some(m - s);
        }
    }

    MacdOutput {
        macd: macd_line,
        signal: signal_line,
        histogram,
    }
}

/// EMA over an [`IndicatorSeries`], preserving alignment (skips leading `None` only).
fn ema_optional(series: &[Option<f64>], period: usize) -> IndicatorSeries {
    if period == 0 {
        return Vec::new();
    }
    let n = series.len();
    let mut out = vec![None; n];
    let Some(start) = series.iter().position(Option::is_some) else {
        return out;
    };
    let compact: Vec<f64> = series[start..].iter().filter_map(|v| *v).collect();
    if compact.len() < period {
        return out;
    }
    let ema_compact = ema(&compact, period);
    for (j, v) in ema_compact.into_iter().enumerate() {
        out[start + j] = v;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::test_util::{assert_macd_matches_fixture, fixture_closes};

    #[test]
    fn macd_histogram_is_macd_minus_signal() {
        let closes = fixture_closes();
        let out = macd(&closes, 12, 26, 9);
        for i in 0..closes.len() {
            if let (Some(m), Some(s), Some(h)) = (out.macd[i], out.signal[i], out.histogram[i]) {
                assert!((h - (m - s)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn macd_matches_fixture() {
        let closes = fixture_closes();
        let got = macd(&closes, 12, 26, 9);
        assert_macd_matches_fixture(&got, "indicators_macd_12_26_9.json");
    }
}
