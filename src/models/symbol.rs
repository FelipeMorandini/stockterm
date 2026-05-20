//! Symbol normalization and classification (Issue #23 / SPEC §43; metadata §44.2).

/// Map Unicode dash characters to ASCII `-` (Yahoo `BTC-USD` chart paths).
fn normalize_symbol_dash(c: char) -> char {
    match c {
        '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        c => c,
    }
}

/// Trim, drop whitespace, normalize dashes, and uppercase ticker input.
///
/// Yahoo chart URLs return **404** when the path contains spaces (e.g. `BTC - USD`);
/// compacting to `BTC-USD` avoids that class of failures. Used by the app layer and
/// [`crate::api::symbol::resolve_provider_symbol`] for Yahoo HTTP paths.
pub fn normalize_symbol(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let compact: String = t
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(normalize_symbol_dash)
        .collect();
    if compact.is_empty() {
        return None;
    }
    Some(compact.to_ascii_uppercase())
}

/// Asset class inferred from a normalized ticker string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Equity,
    Crypto,
    Fx,
    Unknown,
}

const CRYPTO_QUOTE_SUFFIXES: &[&str] = &["-USD", "-USDT", "-EUR", "-GBP", "-BTC"];

fn is_crypto_symbol_heuristic(sym: &str) -> bool {
    CRYPTO_QUOTE_SUFFIXES.iter().any(|suffix| sym.ends_with(suffix))
}

fn is_fx_symbol(sym: &str) -> bool {
    sym.ends_with("=X") || sym.contains('/')
}

fn is_equity_symbol(sym: &str) -> bool {
    if sym.is_empty() || sym.len() > 12 {
        return false;
    }
    let mut chars = sym.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

/// Map Yahoo / Polygon instrument type strings to [`SymbolKind`] (Issue #158 / §44.2).
///
/// Unknown values return [`SymbolKind::Unknown`] so callers can fall back to heuristics.
pub fn classify_from_instrument_type(type_str: &str) -> SymbolKind {
    let upper = type_str.trim().to_ascii_uppercase();
    if upper.is_empty() {
        return SymbolKind::Unknown;
    }
    if upper == "CRYPTOCURRENCY" {
        return SymbolKind::Crypto;
    }
    if upper == "CURRENCY" || upper == "CURRENCYPAIRS" {
        return SymbolKind::Fx;
    }
    if matches!(
        upper.as_str(),
        "EQUITY" | "ETF" | "MUTUALFUND" | "INDEX" | "OPTION"
    ) {
        return SymbolKind::Equity;
    }
    SymbolKind::Unknown
}

/// Classify a normalized ticker (uppercase, trimmed) using string heuristics only.
///
/// Plain short tickers such as **`BTC`** are **not** treated as crypto here — use
/// [`classify_symbol_with_hint`] when Yahoo **`quoteType`** metadata is available.
pub fn classify_symbol(sym: &str) -> SymbolKind {
    classify_symbol_heuristic(sym)
}

fn classify_symbol_heuristic(sym: &str) -> SymbolKind {
    let sym = sym.trim();
    if sym.is_empty() {
        return SymbolKind::Unknown;
    }
    if !sym.is_ascii() {
        return SymbolKind::Unknown;
    }
    let upper = sym.to_uppercase();
    if is_fx_symbol(&upper) {
        return SymbolKind::Fx;
    }
    if is_crypto_symbol_heuristic(&upper) {
        return SymbolKind::Crypto;
    }
    if is_equity_symbol(&upper) {
        return SymbolKind::Equity;
    }
    SymbolKind::Unknown
}

/// Prefer provider metadata when present; otherwise [`classify_symbol`] heuristics.
pub fn classify_symbol_with_hint(sym: &str, instrument_type: Option<&str>) -> SymbolKind {
    if let Some(t) = instrument_type {
        let from_meta = classify_from_instrument_type(t);
        if from_meta != SymbolKind::Unknown {
            return from_meta;
        }
    }
    classify_symbol_heuristic(sym)
}

#[cfg(test)]
mod tests {
    use super::{
        classify_from_instrument_type, classify_symbol, classify_symbol_with_hint,
        normalize_symbol, SymbolKind,
    };

    #[test]
    fn normalize_symbol_trims_and_uppercases() {
        assert_eq!(
            normalize_symbol("  aapl  ").as_deref(),
            Some("AAPL")
        );
        assert_eq!(normalize_symbol("   "), None);
        assert_eq!(normalize_symbol(""), None);
    }

    #[test]
    fn normalize_symbol_compacts_crypto_pair_whitespace() {
        assert_eq!(
            normalize_symbol("btc - usd").as_deref(),
            Some("BTC-USD")
        );
        assert_eq!(
            normalize_symbol("  BTC-USD  ").as_deref(),
            Some("BTC-USD")
        );
    }

    #[test]
    fn classify_symbol_btc_usd_is_crypto() {
        assert_eq!(classify_symbol("BTC-USD"), SymbolKind::Crypto);
    }

    #[test]
    fn classify_symbol_btc_short_is_equity_without_metadata() {
        assert_eq!(classify_symbol("BTC"), SymbolKind::Equity);
        assert_eq!(classify_symbol("ETH"), SymbolKind::Equity);
    }

    #[test]
    fn classify_symbol_aapl_is_equity() {
        assert_eq!(classify_symbol("AAPL"), SymbolKind::Equity);
    }

    #[test]
    fn classify_symbol_eurusd_x_is_fx() {
        assert_eq!(classify_symbol("EURUSD=X"), SymbolKind::Fx);
    }

    #[test]
    fn classify_symbol_empty_is_unknown() {
        assert_eq!(classify_symbol(""), SymbolKind::Unknown);
        assert_eq!(classify_symbol("   "), SymbolKind::Unknown);
    }

    #[test]
    fn classify_symbol_brk_dot_b_is_equity() {
        assert_eq!(classify_symbol("BRK.B"), SymbolKind::Equity);
    }

    #[test]
    fn classify_symbol_eth_usdt_is_crypto() {
        assert_eq!(classify_symbol("ETH-USDT"), SymbolKind::Crypto);
    }

    #[test]
    fn classify_from_instrument_type_cryptocurrency() {
        assert_eq!(
            classify_from_instrument_type("CRYPTOCURRENCY"),
            SymbolKind::Crypto
        );
    }

    #[test]
    fn classify_from_instrument_type_etf_is_equity() {
        assert_eq!(classify_from_instrument_type("ETF"), SymbolKind::Equity);
    }

    #[test]
    fn classify_from_instrument_type_currency_is_fx() {
        assert_eq!(classify_from_instrument_type("CURRENCY"), SymbolKind::Fx);
    }

    #[test]
    fn classify_symbol_with_hint_btc_etf_overrides_short_ticker() {
        assert_eq!(
            classify_symbol_with_hint("BTC", Some("ETF")),
            SymbolKind::Equity
        );
    }

    #[test]
    fn classify_symbol_with_hint_btc_usd_cryptocurrency() {
        assert_eq!(
            classify_symbol_with_hint("BTC-USD", Some("CRYPTOCURRENCY")),
            SymbolKind::Crypto
        );
    }
}
