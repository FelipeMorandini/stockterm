//! Issue #16 / SPEC §23 — substring filter for portfolio + watchlist symbol columns.

/// Upper bound on in-memory filter length (§23.5).
pub const MAX_FILTER_QUERY_LEN: usize = 64;

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

/// Returns backing indices `0..symbols.len()` whose symbol contains `query` (ASCII case-insensitive).
/// Empty `query` yields all indices in order.
pub fn filter_symbol_indices(symbols: &[impl AsRef<str>], query: &str) -> Vec<usize> {
    filter_row_indices(symbols.len(), |i| symbols[i].as_ref(), query)
}

/// Title suffix for Holdings / Watchlist blocks when a filter is active (§23.6).
pub fn filter_title_suffix(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let q: String = query
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let q = q.replace('"', "'");
    format!(r#" (filter: "{q}")"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_all_indices() {
        let s = vec!["AAPL", "MSFT"];
        assert_eq!(filter_symbol_indices(&s, ""), vec![0, 1]);
    }

    #[test]
    fn aa_matches_aapl() {
        let s = vec!["AAPL", "MSFT"];
        assert_eq!(filter_symbol_indices(&s, "aa"), vec![0]);
    }

    #[test]
    fn case_insensitive() {
        let s = vec!["aapl", "GOOGL"];
        assert_eq!(filter_symbol_indices(&s, "AA"), vec![0]);
        assert_eq!(filter_symbol_indices(&s, "oo"), vec![1]);
    }

    #[test]
    fn no_match_empty_vec() {
        let s = vec!["AAPL", "MSFT"];
        assert!(filter_symbol_indices(&s, "zzz").is_empty());
    }

    #[test]
    fn multi_row_mixed() {
        let s = vec!["AAPL", "AMZN", "MSFT"];
        assert_eq!(filter_symbol_indices(&s, "a"), vec![0, 1]);
        assert_eq!(filter_symbol_indices(&s, "ms"), vec![2]);
    }

    #[test]
    fn filter_row_indices_matches_slice_helper() {
        let s = vec!["AAPL", "MSFT", "GOOGL"];
        assert_eq!(
            filter_row_indices(s.len(), |i| s[i], "ms"),
            filter_symbol_indices(&s, "ms")
        );
    }

    #[test]
    fn filter_title_suffix_strips_control_chars() {
        let s = filter_title_suffix("A\u{7}B");
        assert!(!s.contains('\u{7}'));
        assert!(s.contains("AB"));
    }
}
