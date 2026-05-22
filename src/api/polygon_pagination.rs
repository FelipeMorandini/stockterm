//! Shared Polygon REST pagination helpers (Issue #176 / SPEC §53.1).

use crate::api::error::{ProviderError, ProviderResult};
use crate::models::historical::{HistoricalData, HistoricalResponse};

/// Maximum aggregate pages per historical fetch (Issue #176 / §53.1.2).
pub const POLYGON_HISTORICAL_MAX_PAGES: usize = 20;

const POLYGON_API_ORIGIN: &str = "https://api.polygon.io";

/// Rejects pagination URLs that are not Polygon REST (SSRF guard).
pub(crate) fn validate_polygon_next_url(next: &str) -> ProviderResult<String> {
    if !next.starts_with(POLYGON_API_ORIGIN) {
        return Err(ProviderError::ApiMessage(
            "Polygon pagination URL rejected".into(),
        ));
    }
    let rest = &next[POLYGON_API_ORIGIN.len()..];
    if !rest.is_empty() {
        let boundary = rest.as_bytes()[0];
        if boundary != b'/' && boundary != b'?' {
            return Err(ProviderError::ApiMessage(
                "Polygon pagination URL rejected".into(),
            ));
        }
    }
    Ok(next.to_string())
}

/// Appends one aggregates page into `merged`.
pub(crate) fn extend_historical_results(
    merged: &mut Vec<HistoricalData>,
    page: &HistoricalResponse,
) {
    merged.extend_from_slice(&page.results);
}

/// Merges multiple pages into one envelope (used by fetch loop and unit tests).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn merge_historical_pages(pages: &[HistoricalResponse]) -> HistoricalResponse {
    let mut merged_results: Vec<HistoricalData> = Vec::new();
    let mut last = HistoricalResponse::default();
    for page in pages {
        extend_historical_results(&mut merged_results, page);
        last = page.clone();
    }
    let merged_len = merged_results.len() as u32;
    HistoricalResponse {
        results: merged_results,
        count: merged_len,
        results_count: last.results_count,
        next_url: last.next_url,
        status: last.status,
        ticker: last.ticker,
        request_id: last.request_id,
        error: None,
    }
}

/// URL path for debug logging (strips query, including `apiKey`).
pub(crate) fn polygon_url_for_log(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::error::ProviderError;
    use crate::models::historical::HistoricalResponse;

    #[test]
    fn validate_polygon_next_url_rejects_foreign_host() {
        let err = validate_polygon_next_url("https://evil.example/next").unwrap_err();
        assert!(matches!(err, ProviderError::ApiMessage(_)));
    }

    #[test]
    fn validate_polygon_next_url_rejects_subdomain_suffix() {
        let err =
            validate_polygon_next_url("https://api.polygon.io.evil.com/v2/next").unwrap_err();
        assert!(matches!(err, ProviderError::ApiMessage(_)));
    }

    #[test]
    fn validate_polygon_next_url_accepts_polygon_host() {
        let url = validate_polygon_next_url(
            "https://api.polygon.io/v3/snapshot/options/AAPL?cursor=abc",
        )
        .unwrap();
        assert!(url.contains("api.polygon.io"));
    }

    #[test]
    fn merge_historical_pages_concatenates_results() {
        let bar = crate::models::historical::HistoricalData {
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 0.0,
            t: 0,
            vw: 0.0,
            n: None,
        };
        let page1 = HistoricalResponse {
            results: vec![bar.clone(), bar.clone()],
            results_count: 3,
            next_url: Some("https://api.polygon.io/next".into()),
            ..Default::default()
        };
        let page2 = HistoricalResponse {
            results: vec![bar],
            results_count: 3,
            next_url: None,
            ..Default::default()
        };
        let merged = merge_historical_pages(&[page1, page2]);
        assert_eq!(merged.results.len(), 3);
        assert!(merged.next_url.is_none());
        assert_eq!(merged.results_count, 3);
    }
}
