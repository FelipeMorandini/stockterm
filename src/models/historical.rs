use serde::Deserialize;

/// Polygon aggregates JSON envelope (Yahoo chart adapter leaves pagination fields at default).
#[derive(Deserialize, Debug, Default, Clone)]
pub struct HistoricalResponse {
    #[serde(default)]
    pub ticker: String,
    #[serde(default)]
    pub results: Vec<HistoricalData>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub count: u32,
    /// Polygon `resultsCount` — total matching aggregates for the query (Issue #65 / §52.1.2).
    #[serde(default, alias = "resultsCount")]
    pub results_count: u32,
    /// Present when more pages exist beyond `limit` (Issue #65 / §52.1.2).
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl HistoricalResponse {
    /// User-visible API failure when HTTP was 200 but JSON indicates an error (Polygon).
    pub fn api_error_message(&self) -> Option<String> {
        if let Some(e) = &self.error {
            return Some(e.clone());
        }
        let s = self.status.as_str();
        if !s.is_empty() && s != "OK" && s != "DELAYED" {
            return Some(format!("Polygon status: {s}"));
        }
        None
    }
}

/// True when a single Polygon aggregates page is likely incomplete (Issue #65 / §52.1.2).
pub fn polygon_page_truncated(resp: &HistoricalResponse, _requested_limit: u32) -> bool {
    resp.next_url.as_ref().is_some_and(|s| !s.is_empty())
        || (resp.results_count > 0 && resp.results_count as usize > resp.results.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_page_truncated_false_when_fully_merged() {
        let bar = HistoricalData {
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 0.0,
            t: 0,
            vw: 0.0,
            n: None,
        };
        let resp = HistoricalResponse {
            results_count: 3,
            results: vec![bar; 3],
            next_url: None,
            ..Default::default()
        };
        assert!(!polygon_page_truncated(&resp, 500));
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HistoricalData {
    pub o: f64,         // Open
    pub h: f64,         // High
    pub l: f64,         // Low
    pub c: f64,         // Close
    pub v: f64,         // Volume (Polygon may return fractional values)
    /// Bar timestamp in **Unix milliseconds (UTC)**, per `docs/SPEC.md` §65.
    ///
    /// Yahoo `chart_to_historical` multiplies `t_sec * 1_000`. Polygon `/v2/aggs/...`
    /// returns ms natively. Consumers may divide by `1_000.0` for seconds-based axes
    /// (e.g. ratatui `Chart::bounds`) but MUST NOT branch on the magnitude.
    pub t: u64,
    pub vw: f64,        // Volume weighted average price
    pub n: Option<u64>, // Number of transactions
}
