//! Provider HTTP symbol resolution (Issues #157 / #158 / SPEC §44.1; §45.2 #161).

use crate::config::MarketProviderKind;
use crate::models::symbol::{is_crypto_symbol_heuristic, normalize_symbol};

/// User/config canonical symbol (trim, dash normalize, uppercase).
pub fn user_symbol(s: &str) -> Option<String> {
    normalize_symbol(s)
}

/// Polygon crypto: `BTC-USD` → `X:BTCUSD` when the normalized user symbol matches crypto heuristics.
fn polygon_crypto_wire_from_user(user: &str) -> Option<String> {
    if !is_crypto_symbol_heuristic(user) {
        return None;
    }
    let body = user.replace('-', "");
    if body.is_empty() {
        return None;
    }
    Some(format!("X:{body}"))
}

/// Maps a user-facing ticker to the symbol string used in provider HTTP paths.
///
/// Yahoo uses the normalized user form. Polygon maps hyphenated crypto pairs to the `X:…` namespace.
pub fn resolve_provider_symbol(kind: MarketProviderKind, input: &str) -> String {
    let compact = user_symbol(input).unwrap_or_else(|| input.trim().to_uppercase());
    match kind {
        MarketProviderKind::Yahoo => compact,
        MarketProviderKind::Polygon => polygon_crypto_wire_from_user(&compact).unwrap_or(compact),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MarketProviderKind;

    #[test]
    fn resolve_provider_symbol_yahoo_compacts_crypto_pair() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Yahoo, "  btc-usd "),
            "BTC-USD"
        );
    }

    #[test]
    fn resolve_provider_symbol_polygon_maps_btc_usd_to_x_namespace() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Polygon, "BTC-USD"),
            "X:BTCUSD"
        );
    }

    #[test]
    fn resolve_provider_symbol_polygon_leaves_equity_identity() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Polygon, "aapl"),
            "AAPL"
        );
    }

    #[test]
    fn resolve_provider_symbol_yahoo_unchanged_for_crypto() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Yahoo, "BTC-USD"),
            "BTC-USD"
        );
    }

    #[test]
    fn resolve_provider_symbol_polygon_eth_usd_maps_to_x_namespace() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Polygon, "ETH-USD"),
            "X:ETHUSD"
        );
    }

    #[test]
    fn resolve_provider_symbol_polygon_btcusd_without_hyphen_is_identity() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Polygon, "BTCUSD"),
            "BTCUSD"
        );
    }

    #[test]
    fn resolve_provider_symbol_empty_trim_fallback() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Yahoo, "   "),
            ""
        );
    }
}
