//! Provider HTTP symbol resolution (Issues #157 / #158 / SPEC §44.1).

use crate::config::MarketProviderKind;
use crate::models::symbol::normalize_symbol;

/// User/config canonical symbol (trim, dash normalize, uppercase).
pub fn user_symbol(s: &str) -> Option<String> {
    normalize_symbol(s)
}

/// Maps a user-facing ticker to the symbol string used in provider HTTP paths.
///
/// v1: Yahoo and Polygon both use the normalized user form (compact uppercase).
/// Polygon crypto namespaces (`X:BTCUSD`) are documented only — no auto-translation.
pub fn resolve_provider_symbol(kind: MarketProviderKind, input: &str) -> String {
    let _ = kind;
    user_symbol(input).unwrap_or_else(|| input.trim().to_uppercase())
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
    fn resolve_provider_symbol_polygon_uppercases_equity() {
        assert_eq!(
            resolve_provider_symbol(MarketProviderKind::Polygon, "aapl"),
            "AAPL"
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
