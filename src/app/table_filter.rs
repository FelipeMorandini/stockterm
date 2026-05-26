//! Issue #16 / SPEC §23 — substring filter for portfolio + watchlist symbol columns.
//! Issue #194 / SPEC §69 — optional regex mode + title suffix extensions.

use regex::Regex;

/// Upper bound on in-memory filter length (§23.5).
pub const MAX_FILTER_QUERY_LEN: usize = 64;

const MAX_REGEX_ERROR_DISPLAY: usize = 64;

/// Result of compiling an active regex filter pattern (§69.3.1).
#[derive(Debug, Clone)]
pub struct CompiledTableFilter {
    /// `Some` when `regex_mode` and pattern is non-empty and valid.
    pub compiled_regex: Option<Regex>,
    /// Set when `regex_mode` and compile failed.
    pub regex_error: Option<String>,
}

/// Wrap `pattern` with `(?i)` unless it already starts with `(?` (§69.3.1).
pub fn compile_filter_regex(pattern: &str) -> Result<Regex, String> {
    let wrapped = if pattern.starts_with("(?)") || pattern.starts_with("(?") {
        pattern.to_string()
    } else {
        format!("(?i){pattern}")
    };
    Regex::new(&wrapped).map_err(|e| truncate_regex_error(&e.to_string()))
}

fn truncate_regex_error(msg: &str) -> String {
    if msg.len() <= MAX_REGEX_ERROR_DISPLAY {
        return msg.to_string();
    }
    let mut s: String = msg.chars().take(MAX_REGEX_ERROR_DISPLAY).collect();
    s.push('…');
    s
}

/// Compile when `regex_mode` and `query` is non-empty; otherwise clears error state.
pub fn compile_active_filter(query: &str, regex_mode: bool) -> CompiledTableFilter {
    if !regex_mode || query.is_empty() {
        return CompiledTableFilter {
            compiled_regex: None,
            regex_error: None,
        };
    }
    match compile_filter_regex(query) {
        Ok(re) => CompiledTableFilter {
            compiled_regex: Some(re),
            regex_error: None,
        },
        Err(e) => CompiledTableFilter {
            compiled_regex: None,
            regex_error: Some(format!("invalid regex: {e}")),
        },
    }
}

/// Returns true when `c` may be appended to the filter query in the current mode (§69.3.3).
pub fn filter_query_char_allowed(c: char, regex_mode: bool) -> bool {
    if c.is_alphanumeric() || c == '-' || c == '.' || c == '=' {
        return true;
    }
    if !regex_mode {
        return false;
    }
    matches!(
        c,
        '*' | '+' | '?' | '[' | ']' | '(' | ')' | '|' | '^' | '$' | '\\' | '{' | '}' | ':'
    )
}

/// Row indices `0..len` whose `symbol_at(i)` matches the active filter.
///
/// Empty `query` yields all indices. Invalid regex yields all indices (§69.3.1).
pub(crate) fn filter_row_indices_with_mode<'a>(
    len: usize,
    mut symbol_at: impl FnMut(usize) -> &'a str,
    query: &str,
    regex_mode: bool,
    compiled: &CompiledTableFilter,
) -> Vec<usize> {
    if query.is_empty() {
        return (0..len).collect();
    }
    if regex_mode {
        if compiled.regex_error.is_some() {
            return (0..len).collect();
        }
        let Some(re) = &compiled.compiled_regex else {
            return (0..len).collect();
        };
        return (0..len).filter(|&i| re.is_match(symbol_at(i))).collect();
    }
    filter_row_indices(len, symbol_at, query)
}

/// Row indices `0..len` whose `symbol_at(i)` contains `query` (ASCII case-insensitive substring).
/// Empty `query` yields all indices in order.
///
/// Matching uses [`str::to_ascii_lowercase`] on each symbol and on the query (ticker symbols are
/// expected ASCII; non-ASCII tickers may not match intuitively).
pub(crate) fn filter_row_indices<'a>(
    len: usize,
    mut symbol_at: impl FnMut(usize) -> &'a str,
    query: &str,
) -> Vec<usize> {
    if query.is_empty() {
        return (0..len).collect();
    }
    let needle = query.to_ascii_lowercase();
    (0..len)
        .filter(|&i| symbol_at(i).to_ascii_lowercase().contains(needle.as_str()))
        .collect()
}

/// Returns backing indices `0..symbols.len()` whose symbol matches the active filter.
pub fn filter_symbol_indices_with_mode(
    symbols: &[impl AsRef<str>],
    query: &str,
    regex_mode: bool,
    compiled: &CompiledTableFilter,
) -> Vec<usize> {
    if !regex_mode {
        return filter_symbol_indices(symbols, query);
    }
    filter_row_indices_with_mode(
        symbols.len(),
        |i| symbols[i].as_ref(),
        query,
        true,
        compiled,
    )
}

/// Returns backing indices `0..symbols.len()` whose symbol contains `query` (ASCII case-insensitive).
/// Empty `query` yields all indices in order.
pub fn filter_symbol_indices(symbols: &[impl AsRef<str>], query: &str) -> Vec<usize> {
    filter_row_indices(symbols.len(), |i| symbols[i].as_ref(), query)
}

/// Title suffix for Holdings / Watchlist blocks when a filter is active (§23.6 / §69.3.4).
pub fn filter_title_suffix(query: &str, regex_mode: bool, regex_error: Option<&str>) -> String {
    if query.is_empty() {
        return String::new();
    }
    let q: String = query.chars().filter(|c| !c.is_control()).collect();
    let q = q.replace('"', "'");
    if regex_mode {
        if regex_error.is_some() {
            return format!(r#" (filter: /{q}/ regex — invalid)"#);
        }
        return format!(r#" (filter: /{q}/ regex)"#);
    }
    format!(r#" (filter: "{q}")"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_all_indices() {
        let s = vec!["AAPL", "MSFT"];
        let c = compile_active_filter("", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "", false, &c),
            vec![0, 1]
        );
    }

    #[test]
    fn aa_matches_aapl() {
        let s = vec!["AAPL", "MSFT"];
        let c = compile_active_filter("aa", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "aa", false, &c),
            vec![0]
        );
    }

    #[test]
    fn case_insensitive() {
        let s = vec!["aapl", "GOOGL"];
        let c = compile_active_filter("AA", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "AA", false, &c),
            vec![0]
        );
        let c2 = compile_active_filter("oo", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "oo", false, &c2),
            vec![1]
        );
    }

    #[test]
    fn no_match_empty_vec() {
        let s = vec!["AAPL", "MSFT"];
        let c = compile_active_filter("zzz", false);
        assert!(filter_symbol_indices_with_mode(&s, "zzz", false, &c).is_empty());
    }

    #[test]
    fn multi_row_mixed() {
        let s = vec!["AAPL", "AMZN", "MSFT"];
        let c = compile_active_filter("a", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "a", false, &c),
            vec![0, 1]
        );
        let c2 = compile_active_filter("ms", false);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "ms", false, &c2),
            vec![2]
        );
    }

    #[test]
    fn filter_row_indices_matches_slice_helper() {
        let s = vec!["AAPL", "MSFT", "GOOGL"];
        let c = compile_active_filter("ms", false);
        assert_eq!(
            filter_row_indices_with_mode(s.len(), |i| s[i], "ms", false, &c),
            filter_symbol_indices(&s, "ms")
        );
    }

    #[test]
    fn filter_title_suffix_strips_control_chars() {
        let s = filter_title_suffix("A\u{7}B", false, None);
        assert!(!s.contains('\u{7}'));
        assert!(s.contains("AB"));
    }

    #[test]
    fn regex_caret_a_matches_aapl_not_ba() {
        let s = vec!["AAPL", "BA", "MSFT"];
        let compiled = compile_active_filter("^A", true);
        assert!(compiled.regex_error.is_none());
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "^A", true, &compiled),
            vec![0]
        );
    }

    #[test]
    fn invalid_regex_returns_all_indices() {
        let s = vec!["AAPL", "MSFT"];
        let compiled = compile_active_filter("[", true);
        assert!(compiled.regex_error.is_some());
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "[", true, &compiled),
            vec![0, 1]
        );
    }

    #[test]
    fn regex_usd_suffix() {
        let s = vec!["AAPL", "BTC-USD"];
        let compiled = compile_active_filter("USD$", true);
        assert_eq!(
            filter_symbol_indices_with_mode(&s, "USD$", true, &compiled),
            vec![1]
        );
    }

    #[test]
    fn compile_adds_case_insensitive_flag() {
        let re = compile_filter_regex("^ms").unwrap();
        assert!(re.is_match("msft"));
    }

    #[test]
    fn filter_query_char_allowed_regex_metas() {
        assert!(filter_query_char_allowed('*', true));
        assert!(!filter_query_char_allowed('*', false));
    }
}
