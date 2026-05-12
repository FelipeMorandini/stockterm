//! Single GET + status / body handling for market-data providers (SPEC §19.4).

use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;

use crate::api::error::{map_reqwest, ProviderError, ProviderResult};

/// Max bytes read from an error response body before building a snippet.
const MAX_ERROR_BODY_BYTES: usize = 4096;
/// UTF-8 safe character cap for [`ProviderError::Http::body_snippet`].
const SNIPPET_MAX_CHARS: usize = 256;

#[derive(Debug)]
pub(crate) enum FetchOnceError {
    RateLimited {
        retry_after: Option<Duration>,
        body_snippet: Option<String>,
    },
    Fatal(ProviderError),
}

pub(crate) fn parse_retry_after_value(raw: &str) -> Option<Duration> {
    let raw = raw.trim();

    if let Ok(secs) = raw.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // `chrono::DateTime::parse_from_rfc2822` accepts `+0000` offsets; many servers send `… GMT`.
    let rfc2822 = raw
        .strip_suffix(" GMT")
        .map(|s| format!("{} +0000", s.trim_end()))
        .unwrap_or_else(|| raw.to_string());

    if let Ok(dt) = DateTime::parse_from_rfc2822(&rfc2822).or_else(|_| DateTime::parse_from_rfc2822(raw)) {
        let now = Utc::now();
        let target = dt.with_timezone(&Utc);
        if target > now {
            return target.signed_duration_since(now).to_std().ok();
        }
        return Some(Duration::ZERO);
    }

    None
}

pub(crate) fn parse_retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    let raw = headers.get("retry-after")?.to_str().ok()?;
    parse_retry_after_value(raw)
}

fn snippet_if_non_empty(text: &str) -> Option<String> {
    let s: String = text
        .chars()
        .filter(|c| !c.is_control())
        .take(SNIPPET_MAX_CHARS)
        .collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

async fn drain_error_body(resp: reqwest::Response, max_bytes: usize) -> ProviderResult<String> {
    let bytes = resp.bytes().await.map_err(map_reqwest)?;
    let n = bytes.len().min(max_bytes);
    Ok(String::from_utf8_lossy(&bytes[..n]).into_owned())
}

pub(crate) async fn get_text_once(
    client: &reqwest::Client,
    url: &str,
) -> Result<String, FetchOnceError> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| FetchOnceError::Fatal(map_reqwest(e)))?;

    let status = resp.status();
    let headers = resp.headers().clone();
    let code = status.as_u16();

    if code == 429 {
        let retry_after = parse_retry_after_header(&headers);
        let text = drain_error_body(resp, MAX_ERROR_BODY_BYTES)
            .await
            .map_err(FetchOnceError::Fatal)?;
        return Err(FetchOnceError::RateLimited {
            retry_after,
            body_snippet: snippet_if_non_empty(&text),
        });
    }

    if !status.is_success() {
        let text = drain_error_body(resp, MAX_ERROR_BODY_BYTES)
            .await
            .map_err(FetchOnceError::Fatal)?;
        return Err(FetchOnceError::Fatal(ProviderError::Http {
            status: code,
            url: url.to_string(),
            body_snippet: snippet_if_non_empty(&text),
        }));
    }

    let text = resp.text().await.map_err(|e| FetchOnceError::Fatal(map_reqwest(e)))?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn parse_retry_after_integer_seconds() {
        let mut h = HeaderMap::new();
        h.insert("retry-after", HeaderValue::from_static("120"));
        assert_eq!(
            parse_retry_after_header(&h),
            Some(Duration::from_secs(120))
        );
    }

    #[test]
    fn parse_retry_after_malformed_returns_none() {
        let mut h = HeaderMap::new();
        h.insert("retry-after", HeaderValue::from_static("not-a-number"));
        assert!(parse_retry_after_header(&h).is_none());
    }

    #[test]
    fn parse_retry_after_future_http_date_gmt() {
        let d = parse_retry_after_value("Wed, 01 Jul 2099 12:00:00 GMT").expect("future");
        assert!(d > Duration::ZERO);
    }

    #[test]
    fn parse_retry_after_past_http_date_yields_zero() {
        // RFC 2822 shape per chrono (`DateTime::parse_from_rfc2822` examples use `+0000`, not `GMT`).
        assert_eq!(
            parse_retry_after_value("Thu, 1 Jan 1970 00:00:00 +0000"),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn parse_retry_after_missing() {
        assert!(parse_retry_after_header(&HeaderMap::new()).is_none());
    }
}
