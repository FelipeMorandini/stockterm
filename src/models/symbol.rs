//! Symbol normalization and classification (Issue #23 / SPEC §43; metadata §44.2; Unicode §67 / #79; config load §73 / #204).

use crate::models::alerts::Alert;
use crate::models::dashboard::DashboardDefinition;
use crate::models::portfolio::PortfolioItem;
use unicode_normalization::UnicodeNormalization;

/// Summary of config symbol migration performed during load (Issue #204 / §73).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SymbolCanonicalizeReport {
    pub watchlist_rewritten: usize,
    pub watchlist_deduped: usize,
    pub portfolio_rewritten: usize,
    pub portfolio_deduped: usize,
    pub alerts_rewritten: usize,
    pub default_symbol_rewritten: bool,
    pub last_symbol_rewritten: bool,
    pub dashboard_symbol_overrides_rewritten: usize,
    pub invalid_dropped: usize,
}

impl SymbolCanonicalizeReport {
    /// True when any persisted symbol field was rewritten, deduped, or dropped.
    pub fn any_changes(self) -> bool {
        self.watchlist_rewritten > 0
            || self.watchlist_deduped > 0
            || self.portfolio_rewritten > 0
            || self.portfolio_deduped > 0
            || self.alerts_rewritten > 0
            || self.default_symbol_rewritten
            || self.last_symbol_rewritten
            || self.dashboard_symbol_overrides_rewritten > 0
            || self.invalid_dropped > 0
    }
}

fn out_contains_equivalent(out: &[String], sym: &str) -> bool {
    out.iter().any(|x| symbols_equivalent(x, sym))
}

/// Rewrite [`watchlist`] to §67 canonical form; dedupe equivalent symbols (first wins).
pub fn canonicalize_watchlist_symbols(
    watchlist: &mut Vec<String>,
    report: &mut SymbolCanonicalizeReport,
) {
    let raw = std::mem::take(watchlist);
    let mut out = Vec::with_capacity(raw.len());
    for raw_sym in raw {
        let Some(c) = normalize_symbol(&raw_sym) else {
            report.invalid_dropped += 1;
            continue;
        };
        if out_contains_equivalent(&out, &c) {
            report.watchlist_deduped += 1;
            continue;
        }
        if c != raw_sym {
            report.watchlist_rewritten += 1;
        }
        out.push(c);
    }
    *watchlist = out;
}

/// Rewrite portfolio row symbols; drop later rows equivalent to an earlier symbol.
pub fn canonicalize_portfolio_symbols(
    portfolio: &mut Vec<PortfolioItem>,
    report: &mut SymbolCanonicalizeReport,
) {
    let raw = std::mem::take(portfolio);
    let mut out = Vec::with_capacity(raw.len());
    for mut item in raw {
        let Some(c) = normalize_symbol(&item.symbol) else {
            report.invalid_dropped += 1;
            continue;
        };
        if out
            .iter()
            .any(|row: &PortfolioItem| symbols_equivalent(&row.symbol, &c))
        {
            report.portfolio_deduped += 1;
            continue;
        }
        if item.symbol != c {
            report.portfolio_rewritten += 1;
            item.symbol = c;
        }
        out.push(item);
    }
    if report.portfolio_deduped > 0 {
        tracing::warn!(
            dropped = report.portfolio_deduped,
            "removed duplicate portfolio rows while canonicalizing symbols on config load"
        );
    }
    *portfolio = out;
}

/// Rewrite alert symbols to canonical form (alerts are not deduped).
pub fn canonicalize_alert_symbols(alerts: &mut Vec<Alert>, report: &mut SymbolCanonicalizeReport) {
    let mut invalid_alerts = 0usize;
    for alert in alerts.iter_mut() {
        let raw = alert.symbol.clone();
        let Some(c) = normalize_symbol(&raw) else {
            report.invalid_dropped += 1;
            invalid_alerts += 1;
            alert.symbol.clear();
            continue;
        };
        if c != raw {
            report.alerts_rewritten += 1;
            alert.symbol = c;
        }
    }
    alerts.retain(|a| !a.symbol.is_empty());
    if invalid_alerts > 0 {
        tracing::warn!(
            dropped = invalid_alerts,
            "removed invalid alerts while canonicalizing symbols on config load"
        );
    }
}

/// Rewrite optional session/default symbol fields on config.
pub fn canonicalize_optional_symbol_field(
    field: &mut Option<String>,
    report: &mut SymbolCanonicalizeReport,
    track_rewrite: impl FnOnce(&mut SymbolCanonicalizeReport),
) {
    let Some(raw) = field.take() else {
        return;
    };
    let Some(c) = normalize_symbol(&raw) else {
        report.invalid_dropped += 1;
        return;
    };
    if c != raw {
        track_rewrite(report);
    }
    *field = Some(c);
}

/// Rewrite non-empty default symbol string.
pub fn canonicalize_default_symbol(
    default_symbol: &mut String,
    report: &mut SymbolCanonicalizeReport,
) {
    if default_symbol.is_empty() {
        return;
    }
    let raw = std::mem::take(default_symbol);
    let Some(c) = normalize_symbol(&raw) else {
        report.invalid_dropped += 1;
        return;
    };
    if c != raw {
        report.default_symbol_rewritten = true;
    }
    *default_symbol = c;
}

/// Rewrite dashboard pane symbol overrides.
pub fn canonicalize_dashboard_symbol_overrides(
    dashboards: &mut [DashboardDefinition],
    report: &mut SymbolCanonicalizeReport,
) {
    for def in dashboards.iter_mut() {
        for pane in def.panes.iter_mut() {
            let Some(raw) = pane.options.symbol.take() else {
                continue;
            };
            let Some(c) = normalize_symbol(&raw) else {
                report.invalid_dropped += 1;
                continue;
            };
            if c != raw {
                report.dashboard_symbol_overrides_rewritten += 1;
            }
            pane.options.symbol = Some(c);
        }
    }
}

/// Rewrite all persisted ticker fields in config (Issue #204 / §73).
pub fn canonicalize_persisted_symbol_fields(
    watchlist: &mut Vec<String>,
    portfolio: &mut Vec<PortfolioItem>,
    alerts: &mut Vec<Alert>,
    default_symbol: &mut String,
    last_symbol: &mut Option<String>,
    dashboards: &mut [DashboardDefinition],
) -> SymbolCanonicalizeReport {
    let mut report = SymbolCanonicalizeReport::default();
    canonicalize_watchlist_symbols(watchlist, &mut report);
    canonicalize_portfolio_symbols(portfolio, &mut report);
    canonicalize_alert_symbols(alerts, &mut report);
    canonicalize_default_symbol(default_symbol, &mut report);
    canonicalize_optional_symbol_field(last_symbol, &mut report, |r| {
        r.last_symbol_rewritten = true;
    });
    canonicalize_dashboard_symbol_overrides(dashboards, &mut report);
    report
}

/// Map Unicode dash characters to ASCII `-` (Yahoo `BTC-USD` chart paths).
fn normalize_symbol_dash(c: char) -> char {
    match c {
        '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        c => c,
    }
}

/// Allowed in user/config ticker strings after NFC compaction (§67.4.2).
fn is_allowed_symbol_char(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit() || c == '-' || c == '.' || c == '='
}

fn symbol_nfc_compact(s: &str) -> Option<String> {
    let compact = symbol_compact_input(s)?;
    let nfc: String = compact.nfc().collect();
    if nfc.is_empty() {
        None
    } else {
        Some(nfc)
    }
}

/// Trim, drop whitespace, normalize dashes — shared by [`normalize_symbol`] and [`symbols_equivalent`].
fn symbol_compact_input(s: &str) -> Option<String> {
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
        None
    } else {
        Some(compact)
    }
}

fn symbol_case_fold_key(nfc_compact: &str) -> String {
    let mut out = String::with_capacity(nfc_compact.len());
    for c in nfc_compact.chars() {
        match c {
            // Rust `to_lowercase` keeps ß; Unicode default case fold maps ß → ss (§67).
            '\u{00DF}' | '\u{1E9E}' => out.push_str("ss"),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// True when two ticker strings denote the same instrument under §67 rules.
pub fn symbols_equivalent(a: &str, b: &str) -> bool {
    let (Some(ca), Some(cb)) = (symbol_nfc_compact(a), symbol_nfc_compact(b)) else {
        return false;
    };
    if !ca.chars().all(is_allowed_symbol_char) || !cb.chars().all(is_allowed_symbol_char) {
        return false;
    }
    symbol_case_fold_key(&ca) == symbol_case_fold_key(&cb)
}

/// Trim, drop whitespace, normalize dashes, NFC, and uppercase ticker input.
///
/// Yahoo chart URLs return **404** when the path contains spaces (e.g. `BTC - USD`);
/// compacting to `BTC-USD` avoids that class of failures. Used by the app layer and
/// [`crate::api::symbol::resolve_provider_symbol`] for Yahoo HTTP paths.
///
/// Rejects symbols containing characters outside Unicode letters, digits, `-`, `.`, or `=`.
pub fn normalize_symbol(s: &str) -> Option<String> {
    let nfc = symbol_nfc_compact(s)?;
    if !nfc.chars().all(is_allowed_symbol_char) {
        return None;
    }
    let canonical: String = nfc.chars().flat_map(char::to_uppercase).collect();
    if canonical.is_empty() {
        None
    } else {
        Some(canonical)
    }
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

/// True when a normalized ticker looks like a hyphenated crypto pair (§45.2 / Issue #161).
pub(crate) fn is_crypto_symbol_heuristic(sym: &str) -> bool {
    CRYPTO_QUOTE_SUFFIXES
        .iter()
        .any(|suffix| sym.ends_with(suffix))
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
        canonicalize_dashboard_symbol_overrides, canonicalize_persisted_symbol_fields,
        canonicalize_watchlist_symbols, classify_from_instrument_type, classify_symbol,
        classify_symbol_with_hint, normalize_symbol, symbols_equivalent, SymbolCanonicalizeReport,
        SymbolKind,
    };
    use crate::models::alerts::{Alert, AlertCondition};
    use crate::models::dashboard::{
        DashboardDefinition, DashboardPane, DashboardPaneKind, DashboardPaneOptions,
    };
    use crate::models::portfolio::PortfolioItem;

    #[test]
    fn canonicalize_watchlist_mixed_case_dedupes() {
        let mut watchlist = vec!["aapl".into(), "AAPL".into(), "msft".into()];
        let mut report = SymbolCanonicalizeReport::default();
        canonicalize_watchlist_symbols(&mut watchlist, &mut report);
        assert_eq!(watchlist, vec!["AAPL", "MSFT"]);
        assert_eq!(report.watchlist_rewritten, 2);
        assert_eq!(report.watchlist_deduped, 1);
    }

    #[test]
    fn canonicalize_portfolio_dedupes_equivalent_rows() {
        let mut portfolio = vec![
            PortfolioItem::new("aapl".into(), 1.0, 100.0),
            PortfolioItem::new("AAPL".into(), 2.0, 110.0),
            PortfolioItem::new("MSFT".into(), 1.0, 200.0),
        ];
        let mut report = SymbolCanonicalizeReport::default();
        super::canonicalize_portfolio_symbols(&mut portfolio, &mut report);
        assert_eq!(portfolio.len(), 2);
        assert_eq!(portfolio[0].symbol, "AAPL");
        assert_eq!(portfolio[0].shares, 1.0);
        assert_eq!(portfolio[1].symbol, "MSFT");
        assert_eq!(report.portfolio_deduped, 1);
    }

    #[test]
    fn canonicalize_alerts_rewrite_only_no_dedup() {
        let mut alerts = vec![
            Alert::new("msft".into(), AlertCondition::Above, 1.0),
            Alert::new("MSFT".into(), AlertCondition::Below, 2.0),
        ];
        let mut report = SymbolCanonicalizeReport::default();
        super::canonicalize_alert_symbols(&mut alerts, &mut report);
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].symbol, "MSFT");
        assert_eq!(alerts[1].symbol, "MSFT");
        assert_eq!(report.alerts_rewritten, 1);
    }

    #[test]
    fn canonicalize_drops_invalid_watchlist_entry() {
        let mut watchlist = vec!["AAPL".into(), "AA\u{0000}PL".into()];
        let mut report = SymbolCanonicalizeReport::default();
        canonicalize_watchlist_symbols(&mut watchlist, &mut report);
        assert_eq!(watchlist, vec!["AAPL"]);
        assert_eq!(report.invalid_dropped, 1);
    }

    #[test]
    fn canonicalize_watchlist_unicode_dedup() {
        let eszett = "stra\u{00df}e";
        let mut watchlist = vec![eszett.to_string(), "STRASSE".into()];
        let mut report = SymbolCanonicalizeReport::default();
        canonicalize_watchlist_symbols(&mut watchlist, &mut report);
        assert_eq!(watchlist, vec!["STRASSE"]);
        assert_eq!(report.watchlist_deduped, 1);
    }

    #[test]
    fn canonicalize_dashboard_symbol_override() {
        let mut dashboards = vec![DashboardDefinition {
            name: "chart_only".into(),
            rows: 1,
            cols: 1,
            panes: vec![DashboardPane {
                id: "c1".into(),
                kind: DashboardPaneKind::Chart,
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: None,
                options: DashboardPaneOptions {
                    symbol: Some("aapl".into()),
                    ..DashboardPaneOptions::default()
                },
            }],
        }];
        let mut report = SymbolCanonicalizeReport::default();
        canonicalize_dashboard_symbol_overrides(&mut dashboards, &mut report);
        assert_eq!(
            dashboards[0].panes[0].options.symbol.as_deref(),
            Some("AAPL")
        );
        assert_eq!(report.dashboard_symbol_overrides_rewritten, 1);
    }

    #[test]
    fn canonicalize_drops_invalid_alert() {
        let mut alerts = vec![
            Alert::new("MSFT".into(), AlertCondition::Above, 1.0),
            Alert::new("AA\u{0000}PL".into(), AlertCondition::Below, 2.0),
        ];
        let mut report = SymbolCanonicalizeReport::default();
        super::canonicalize_alert_symbols(&mut alerts, &mut report);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].symbol, "MSFT");
        assert_eq!(report.invalid_dropped, 1);
    }

    #[test]
    fn canonicalize_persisted_fields_default_and_last_symbol() {
        let mut watchlist = Vec::new();
        let mut portfolio = Vec::new();
        let mut alerts = Vec::new();
        let mut default_symbol = "btc - usd".to_string();
        let mut last_symbol = Some("aapl".to_string());
        let mut dashboards = Vec::new();
        let report = canonicalize_persisted_symbol_fields(
            &mut watchlist,
            &mut portfolio,
            &mut alerts,
            &mut default_symbol,
            &mut last_symbol,
            &mut dashboards,
        );
        assert!(report.any_changes());
        assert_eq!(default_symbol, "BTC-USD");
        assert_eq!(last_symbol.as_deref(), Some("AAPL"));
    }

    #[test]
    fn normalize_symbol_trims_and_uppercases() {
        assert_eq!(normalize_symbol("  aapl  ").as_deref(), Some("AAPL"));
        assert_eq!(normalize_symbol("   "), None);
        assert_eq!(normalize_symbol(""), None);
    }

    #[test]
    fn normalize_symbol_compacts_crypto_pair_whitespace() {
        assert_eq!(normalize_symbol("btc - usd").as_deref(), Some("BTC-USD"));
        assert_eq!(normalize_symbol("  BTC-USD  ").as_deref(), Some("BTC-USD"));
    }

    #[test]
    fn symbols_equivalent_ascii_case() {
        assert!(symbols_equivalent("aapl", "AAPL"));
        assert!(symbols_equivalent("  msft ", "MSFT"));
        assert!(!symbols_equivalent("AAPL", "MSFT"));
    }

    #[test]
    fn symbols_equivalent_unicode_fold_german_eszett() {
        // Case fold maps ß → ss; canonical normalize uppercases to STRASSE.
        let eszett = "stra\u{00df}e";
        assert!(symbols_equivalent(eszett, "STRASSE"));
        assert_eq!(normalize_symbol(eszett).as_deref(), Some("STRASSE"));
    }

    #[test]
    fn normalize_symbol_rejects_control_chars() {
        assert_eq!(normalize_symbol("AA\u{0000}PL"), None);
    }

    #[test]
    fn normalize_symbol_nfc_composes() {
        let decomposed = "A\u{0301}PL"; // A + combining acute
        let composed = "ÁPL";
        assert_eq!(
            normalize_symbol(decomposed).as_deref(),
            normalize_symbol(composed).as_deref()
        );
        assert!(symbols_equivalent(decomposed, composed));
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
