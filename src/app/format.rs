//! USD price and symbol-kind display helpers (Issue #23 / SPEC §43.2).

use crate::models::symbol::SymbolKind;

/// Max rendered width for table price cells; wider values fall back to two decimals.
const MAX_TABLE_PRICE_CHARS: usize = 14;

fn trim_price_trailing_zeros(mut s: String) -> String {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// Format a USD price for table/detail/chart labels (Issue #23 / SPEC §43.2).
///
/// Non-finite values render as `"—"`. Strings longer than [`MAX_TABLE_PRICE_CHARS`]
/// use `${:.2}` so watchlist columns do not overflow narrow terminals.
pub fn format_usd_price(price: f64) -> String {
    if !price.is_finite() {
        return "—".to_string();
    }
    let abs = price.abs();
    let mut s = if abs >= 100_000.0 {
        format!("${abs:.0}")
    } else if abs >= 1.0 {
        format!("${abs:.2}")
    } else if abs >= 0.01 {
        format!("${abs:.4}")
    } else {
        trim_price_trailing_zeros(format!("${abs:.8}"))
    };
    if s.len() > MAX_TABLE_PRICE_CHARS {
        s = format!("${price:.2}");
    }
    s
}

/// Signed USD delta with explicit `+` for non-negative values.
pub fn format_signed_usd_delta(delta: f64) -> String {
    if !delta.is_finite() {
        return "—".to_string();
    }
    let sign = if delta >= 0.0 { "+" } else { "-" };
    let body = format_usd_price(delta.abs());
    format!("{sign}{}", body.trim_start_matches('$'))
}

/// Short kind label for tables (e.g. `CRYPTO`, `FX`, `EQ`).
pub fn symbol_kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Crypto => "CRYPTO",
        SymbolKind::Fx => "FX",
        SymbolKind::Equity => "EQ",
        SymbolKind::Unknown => "",
    }
}

#[cfg(test)]
mod tests {
    use super::{format_signed_usd_delta, format_usd_price};

    #[test]
    fn format_usd_price_micro_cap() {
        assert_eq!(format_usd_price(0.0000123), "$0.0000123");
    }

    #[test]
    fn format_usd_price_large() {
        assert_eq!(format_usd_price(100_000.45), "$100000");
    }

    #[test]
    fn format_usd_price_nan() {
        assert_eq!(format_usd_price(f64::NAN), "—");
    }

    #[test]
    fn format_usd_price_one_dollar() {
        assert_eq!(format_usd_price(1.0), "$1.00");
    }

    #[test]
    fn format_signed_usd_delta_positive() {
        assert_eq!(format_signed_usd_delta(1.5), "+1.50");
    }

    #[test]
    fn format_signed_usd_delta_negative() {
        assert!(format_signed_usd_delta(-2.25).starts_with('-'));
    }
}
