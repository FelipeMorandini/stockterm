use serde::Deserialize;

/// Polygon `/v2/aggs/ticker/.../range/...` body. The `ticker` field is omitted on some responses
/// (empty results, delayed/error payloads), so it must not be required for deserialization.
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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
}
