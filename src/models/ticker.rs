use serde::Deserialize;

/// Polygon `/v2/aggs/ticker/.../range/...` body. The `ticker` field is omitted on some responses
/// (empty results, delayed/error payloads), so it must not be required for deserialization.
#[derive(Deserialize, Debug, Clone)]
pub struct TickerResponse {
    #[serde(default)]
    pub ticker: String,
    #[serde(default)]
    pub results: Vec<TickerResult>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TickerResult {
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    /// Polygon may return fractional volume on some aggregates.
    #[serde(default)]
    pub v: f64,
    pub t: u64,
}

impl TickerResponse {
    /// Prefer the most recent bar (Polygon may return ascending or descending order).
    pub fn latest_result(&self) -> Option<&TickerResult> {
        self.results.iter().max_by_key(|r| r.t)
    }

    pub fn symbol_or<'a>(&'a self, requested: &'a str) -> &'a str {
        if self.ticker.is_empty() {
            requested
        } else {
            self.ticker.as_str()
        }
    }

    /// User-visible API failure when HTTP was 200 but the JSON indicates an error.
    pub fn api_error_message(&self) -> Option<String> {
        if let Some(e) = &self.error {
            return Some(e.clone());
        }
        let s = self.status.as_str();
        if !s.is_empty() && s != "OK" && s != "DELAYED" {
            return Some(format!("Polygon status: {}", s));
        }
        None
    }
}

/// True when `resp` should be treated as quote data for `requested` (Issue #32 / SPEC §41.1).
///
/// Empty `resp.ticker` matches any requested symbol. Prefer
/// [`ticker_response_matches_symbol_for_session`] when `resp` is the active-session
/// `App::ticker_data` pane (empty ticker must not apply to other alert symbols).
pub fn ticker_response_matches_symbol(resp: &TickerResponse, requested: &str) -> bool {
    resp.ticker.is_empty() || resp.ticker.eq_ignore_ascii_case(requested)
}

/// Session-scoped matcher for `App::ticker_data` and Stock View `resolve_quote`.
///
/// Empty `resp.ticker` matches only when `requested` equals `active_symbol` (Polygon omit case).
pub fn ticker_response_matches_symbol_for_session(
    resp: &TickerResponse,
    requested: &str,
    active_symbol: &str,
) -> bool {
    if resp.ticker.is_empty() {
        requested.eq_ignore_ascii_case(active_symbol)
    } else {
        resp.ticker.eq_ignore_ascii_case(requested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_without_ticker_field() {
        let json = r#"{"queryCount":0,"resultsCount":0,"adjusted":true,"results":[],"status":"OK","request_id":1}"#;
        let r: TickerResponse = serde_json::from_str(json).expect("parse");
        assert!(r.ticker.is_empty());
        assert!(r.results.is_empty());
    }

    #[test]
    fn deserialize_with_results_only() {
        let json = r#"{"results":[{"o":1.0,"h":2.0,"l":0.5,"c":1.5,"v":100,"t":1700000000000}]}"#;
        let r: TickerResponse = serde_json::from_str(json).expect("parse");
        assert!(r.ticker.is_empty());
        assert_eq!(r.latest_result().map(|b| b.c), Some(1.5));
    }

    #[test]
    fn deserialize_fractional_volume() {
        let json = r#"{"results":[{"o":1.0,"h":2.0,"l":0.5,"c":1.5,"v":52692761.275784,"t":1700000000000}]}"#;
        let r: TickerResponse = serde_json::from_str(json).expect("parse");
        assert!((r.results[0].v - 52692761.275784).abs() < 1e-6);
    }

    #[test]
    fn ticker_response_matches_symbol_empty_ticker() {
        let resp = TickerResponse {
            ticker: String::new(),
            results: vec![],
            status: "OK".into(),
            error: None,
        };
        assert!(ticker_response_matches_symbol(&resp, "AAPL"));
        assert!(ticker_response_matches_symbol(&resp, "msft"));
    }

    #[test]
    fn ticker_response_matches_symbol_case_insensitive() {
        let resp = TickerResponse {
            ticker: "Msft".into(),
            results: vec![],
            status: "OK".into(),
            error: None,
        };
        assert!(ticker_response_matches_symbol(&resp, "MSFT"));
        assert!(!ticker_response_matches_symbol(&resp, "AAPL"));
    }

    #[test]
    fn ticker_response_matches_symbol_for_session_empty_ticker_active_only() {
        let resp = TickerResponse {
            ticker: String::new(),
            results: vec![],
            status: "OK".into(),
            error: None,
        };
        assert!(ticker_response_matches_symbol_for_session(&resp, "MSFT", "MSFT"));
        assert!(!ticker_response_matches_symbol_for_session(&resp, "AAPL", "MSFT"));
    }
}
