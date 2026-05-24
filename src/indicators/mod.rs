//! Technical indicators over close price series (Issue #21 / SPEC §46.1).

mod ema;
mod macd;
mod rsi;
mod sma;
pub mod types;

pub use ema::ema;
pub use macd::macd;
pub use rsi::rsi;
pub use sma::sma;
pub use types::{
    IndicatorSeries, MacdOutput, EMA_PERIOD, MACD_FAST, MACD_SIGNAL, MACD_SLOW, RSI_PERIOD,
    SMA_PERIOD,
};

#[cfg(test)]
mod test_util {
    use crate::indicators::types::{IndicatorSeries, MacdOutput};
    use serde::Deserialize;
    use std::fs;
    use std::path::PathBuf;

    const REL_EPS: f64 = 1e-6;
    const ABS_EPS: f64 = 1e-4;

    #[derive(Deserialize)]
    struct SeriesFixture {
        closes: Vec<f64>,
        #[serde(default)]
        period: usize,
        expected: Vec<Option<f64>>,
    }

    #[derive(Deserialize)]
    struct MacdFixture {
        closes: Vec<f64>,
        macd: Vec<Option<f64>>,
        signal: Vec<Option<f64>>,
        histogram: Vec<Option<f64>>,
    }

    pub fn fixture_closes() -> Vec<f64> {
        fixture_path("indicators_sma_20.json")
            .and_then(|p| load_series_fixture(&p).map(|f| f.closes))
            .unwrap_or_else(synthetic_closes)
    }

    fn synthetic_closes() -> Vec<f64> {
        (0..40)
            .map(|i| 40.0 + (i as f64) * 0.5 + ((i % 5) as f64) * 0.1)
            .collect()
    }

    fn fixture_path(name: &str) -> Option<PathBuf> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let p = root.join("tests/fixtures").join(name);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    }

    fn load_series_fixture(path: &PathBuf) -> Option<SeriesFixture> {
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    fn load_macd_fixture(path: &PathBuf) -> Option<MacdFixture> {
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        if !a.is_finite() || !b.is_finite() {
            return a == b;
        }
        let diff = (a - b).abs();
        if diff <= ABS_EPS {
            return true;
        }
        let scale = a.abs().max(b.abs()).max(1.0);
        diff / scale <= REL_EPS
    }

    fn assert_option_series(got: &IndicatorSeries, expected: &[Option<f64>]) {
        assert_eq!(got.len(), expected.len(), "series length mismatch");
        for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
            match (g, e) {
                (None, None) => {}
                (Some(a), Some(b)) => {
                    assert!(approx_eq(*a, *b), "index {i}: got {a}, expected {b}");
                }
                _ => panic!("index {i}: got {g:?}, expected {e:?}"),
            }
        }
    }

    pub fn assert_series_matches_fixture(got: &IndicatorSeries, file: &str, period: usize) {
        let path = fixture_path(file).expect("fixture file");
        let fixture = load_series_fixture(&path).expect("parse fixture");
        assert_eq!(fixture.period, period);
        assert_eq!(fixture.closes, fixture_closes());
        assert_option_series(got, &fixture.expected);
    }

    pub fn assert_macd_matches_fixture(got: &MacdOutput, file: &str) {
        let path = fixture_path(file).expect("fixture file");
        let fixture = load_macd_fixture(&path).expect("parse fixture");
        assert_eq!(fixture.closes, fixture_closes());
        assert_option_series(&got.macd, &fixture.macd);
        assert_option_series(&got.signal, &fixture.signal);
        assert_option_series(&got.histogram, &fixture.histogram);
    }
}
