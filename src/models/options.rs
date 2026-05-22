//! Options chain domain types (Issue #22 / SPEC §48.1).

use serde::{Deserialize, Serialize};

/// Call or put leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionRight {
    Call,
    Put,
}

/// Optional Greeks from the provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionGreeks {
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub vega: Option<f64>,
    pub rho: Option<f64>,
}

/// One listed option contract for a strike/expiry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionContract {
    pub symbol: String,
    pub strike: f64,
    pub right: OptionRight,
    pub expiration_ts: u64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub last: Option<f64>,
    pub volume: Option<u64>,
    pub open_interest: Option<u64>,
    pub implied_volatility: Option<f64>,
    pub greeks: Option<OptionGreeks>,
}

/// One selectable expiration date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expiration {
    pub ts: u64,
    pub label: String,
}

/// Calls and puts for a single expiration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsChainSlice {
    pub underlying: String,
    pub expiration: Expiration,
    pub calls: Vec<OptionContract>,
    pub puts: Vec<OptionContract>,
}

/// Full chain view: expirations list + active slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionsChain {
    pub underlying: String,
    pub expirations: Vec<Expiration>,
    pub selected_expiration_ts: u64,
    pub slice: OptionsChainSlice,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_right_serde_round_trip() {
        let c = OptionRight::Call;
        let j = serde_json::to_string(&c).unwrap();
        assert_eq!(j, "\"call\"");
        let p: OptionRight = serde_json::from_str("\"put\"").unwrap();
        assert_eq!(p, OptionRight::Put);
    }
}
