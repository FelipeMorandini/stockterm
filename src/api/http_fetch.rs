//! Single GET + status / body handling for market-data providers (SPEC §19.4).

use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;

use crate::api::error::{map_reqwest, ProviderError, ProviderResult};

/// Max bytes read from an error response body before building a snippet.
const MAX_ERROR_BODY_BYTES: usize = 4096;
/// UTF-8 safe character cap for [`ProviderError::Http::body_snippet`].
const SNIPPET_MAX_CHARS: usize = 256;
/// Max seconds accepted from `Retry-After` integer or HTTP-date delta (SPEC §19.13.2).
const MAX_RETRY_AFTER_PARSE_SECS: u64 = 86_400;

#[derive(Debug)]
pub(crate) enum FetchOnceError {
    RateLimited {
        retry_after: Option<Duration>,
        body_snippet: Option<String>,
    },
    Fatal(ProviderError),
}

fn clamp_retry_after_duration(d: Duration) -> Duration {
    d.min(Duration::from_secs(MAX_RETRY_AFTER_PARSE_SECS))
}

/// Normalize IMF-style `Retry-After` HTTP-date suffixes (`GMT` / `UTC`, any ASCII case) for chrono.
fn normalize_retry_after_http_date(raw: &str) -> String {
    let s = raw.trim();
    let lower = s.to_ascii_lowercase();
    for suf in [" gmt", " utc"] {
        if lower.ends_with(suf) {
            let cut = s.len() - suf.len();
            return format!("{} +0000", s[..cut].trim_end());
        }
    }
    raw.to_string()
}

pub(crate) fn parse_retry_after_value(raw: &str) -> Option<Duration> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if let Ok(secs) = raw.parse::<u64>() {
        let secs = secs.min(MAX_RETRY_AFTER_PARSE_SECS);
        return Some(Duration::from_secs(secs));
    }

    let normalized = normalize_retry_after_http_date(raw);
    for candidate in [normalized.as_str(), raw] {
        let candidate = candidate.trim();
        if let Ok(dt) = DateTime::parse_from_rfc2822(candidate) {
            let now = Utc::now();
            let target = dt.with_timezone(&Utc);
            let d = if target > now {
                target.signed_duration_since(now).to_std().ok()?
            } else {
                Duration::ZERO
            };
            return Some(clamp_retry_after_duration(d));
        }
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

/// Reads at most `max_bytes` from the body without buffering the full response (SPEC §19.13.1).
async fn drain_error_body(mut resp: reqwest::Response, max_bytes: usize) -> ProviderResult<String> {
    let mut acc: Vec<u8> = Vec::new();
    // One-byte chunks need `max_bytes` polls; allow some empty chunks without a tight loop.
    let max_reads = max_bytes.saturating_mul(2).saturating_add(256).min(10_000);
    let mut reads = 0usize;

    while acc.len() < max_bytes && reads < max_reads {
        reads += 1;
        let chunk = match resp.chunk().await.map_err(map_reqwest)? {
            None => break,
            Some(c) => c,
        };
        if chunk.is_empty() {
            continue;
        }
        let remaining = max_bytes - acc.len();
        if chunk.len() <= remaining {
            acc.extend_from_slice(&chunk);
        } else {
            acc.extend_from_slice(&chunk[..remaining]);
            break;
        }
    }

    Ok(String::from_utf8_lossy(&acc).into_owned())
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
    fn parse_retry_after_integer_clamped_to_24h() {
        assert_eq!(
            parse_retry_after_value("999999999"),
            Some(Duration::from_secs(MAX_RETRY_AFTER_PARSE_SECS))
        );
    }

    #[test]
    fn parse_retry_after_zero_seconds() {
        assert_eq!(parse_retry_after_value("0"), Some(Duration::ZERO));
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
        assert!(d <= Duration::from_secs(MAX_RETRY_AFTER_PARSE_SECS));
    }

    #[test]
    fn parse_retry_after_future_http_date_utc() {
        let d = parse_retry_after_value("Wed, 01 Jul 2099 12:00:00 UTC").expect("future");
        assert!(d > Duration::ZERO);
    }

    #[test]
    fn parse_retry_after_future_http_date_gmt_lowercase() {
        let d = parse_retry_after_value("Wed, 01 Jul 2099 12:00:00 gmt").expect("future");
        assert!(d > Duration::ZERO);
    }

    #[test]
    fn parse_retry_after_past_http_date_yields_zero() {
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

#[cfg(test)]
mod wiremock_drain_tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn huge_error_body_snippet_bounded_without_hang() {
        let huge = "x".repeat(MAX_ERROR_BODY_BYTES + 50_000);
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(403)
                    .append_header("content-type", "text/plain")
                    .set_body_string(huge),
            )
            .mount(&srv)
            .await;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .connect_timeout(crate::api::http::HTTP_CONNECT_TIMEOUT)
            .build()
            .expect("client");

        let url = format!("{}/big", srv.uri());
        let err = get_text_once(&client, &url)
            .await
            .expect_err("403");
        match err {
            FetchOnceError::Fatal(ProviderError::Http { body_snippet, .. }) => {
                let snippet = body_snippet.expect("snippet");
                assert!(snippet.len() <= SNIPPET_MAX_CHARS);
            }
            other => panic!("expected Http fatal, got {other:?}"),
        }
    }
}
