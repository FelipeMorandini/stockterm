//! Structured failures from market-data HTTP layers.

use std::fmt;
use std::time::Duration;

use thiserror::Error;

#[derive(Error)]
pub enum ProviderError {
    #[error("Request timed out")]
    Timeout,
    #[error(
        "HTTP {status} ({display_url}){body_suffix}",
        display_url = url_without_query(.url),
        body_suffix = http_body_suffix(.body_snippet)
    )]
    Http {
        status: u16,
        url: String,
        body_snippet: Option<String>,
    },
    /// HTTP 429 before retries are exhausted; after exhaustion callers map to [`Http`] with status 429.
    #[error("{}", rate_limited_display(.retry_after))]
    RateLimited {
        retry_after: Option<Duration>,
    },
    /// JSON deserialization failure.
    ///
    /// **Caveat (Issue #122 / SPEC §20.15.3):** this variant is *not*
    /// preserved across [`Clone`]. [`serde_json::Error`] is not [`Clone`], so
    /// `<ProviderError as Clone>::clone` lossily maps it to
    /// [`ProviderError::ApiMessage`] with body `"Invalid JSON response: {e}"`.
    /// Callers that want to branch on parse failure MUST do so on the *first*
    /// observation of the error (before it is moved into
    /// [`crate::app::FetchDone`], [`crate::app::app_error::AppError::Provider`],
    /// or any field that may be cloned later).
    ///
    /// The pre-clone parse-failure path renders as `[parse]` on the status
    /// line ([`crate::app::app_error::category_from_provider`]); the post-clone
    /// surface renders as `[api]` and is `Sticky` (per
    /// [`crate::app::app_error::persistence_for_provider`]).
    ///
    /// If a future caller requires structured JSON-failure data to survive
    /// cloning, switch the variant to `Json(std::sync::Arc<serde_json::Error>)`
    /// (or equivalent). That is an opt-in, breaking-API change tracked
    /// separately from Issue #122.
    #[error("Invalid JSON response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    ApiMessage(String),
    #[error("Network error: {0}")]
    Transport(String),
}

/// Lossy [`Clone`] for the [`ProviderError::Json`] arm: see that variant's
/// docs for the rationale and the `Arc<serde_json::Error>` follow-up. All
/// other variants are deep-cloned faithfully (Issue #122 / SPEC §20.15.3).
impl Clone for ProviderError {
    fn clone(&self) -> Self {
        match self {
            ProviderError::Timeout => ProviderError::Timeout,
            ProviderError::Http {
                status,
                url,
                body_snippet,
            } => ProviderError::Http {
                status: *status,
                url: url.clone(),
                body_snippet: body_snippet.clone(),
            },
            ProviderError::RateLimited { retry_after } => ProviderError::RateLimited {
                retry_after: *retry_after,
            },
            ProviderError::Json(e) => {
                ProviderError::ApiMessage(format!("Invalid JSON response: {e}"))
            }
            ProviderError::ApiMessage(s) => ProviderError::ApiMessage(s.clone()),
            ProviderError::Transport(s) => ProviderError::Transport(s.clone()),
        }
    }
}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// Omit query string so Polygon `apiKey=…` (and other secrets) never appear in UI/error strings.
fn url_without_query(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

fn http_body_suffix(body_snippet: &Option<String>) -> String {
    body_snippet
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!(" — {s}"))
        .unwrap_or_default()
}

fn rate_limited_display(retry_after: &Option<Duration>) -> String {
    match retry_after {
        Some(d) if !d.is_zero() => {
            let ms = d.as_millis().max(1);
            if ms < 1000 {
                format!("Rate limited (retry after {ms}ms)")
            } else {
                let secs = d.as_secs();
                let ceil_secs = if d.subsec_nanos() > 0 {
                    secs.saturating_add(1)
                } else {
                    secs
                };
                format!("Rate limited (retry after {ceil_secs}s)")
            }
        }
        _ => "Rate limited".to_string(),
    }
}

/// Redact query strings from [`ProviderError::Http::url`] in `Debug` output (Issue #116 / SPEC §19.13.5).
impl fmt::Debug for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Timeout => f.write_str("Timeout"),
            ProviderError::Http {
                status,
                url,
                body_snippet,
            } => f
                .debug_struct("Http")
                .field("status", status)
                .field("url", &url_without_query(url))
                .field("body_snippet", body_snippet)
                .finish(),
            ProviderError::RateLimited { retry_after } => f
                .debug_struct("RateLimited")
                .field("retry_after", retry_after)
                .finish(),
            ProviderError::Json(err) => f.debug_tuple("Json").field(err).finish(),
            ProviderError::ApiMessage(msg) => f.debug_tuple("ApiMessage").field(msg).finish(),
            ProviderError::Transport(msg) => f.debug_tuple("Transport").field(msg).finish(),
        }
    }
}

pub fn map_reqwest(err: reqwest::Error) -> ProviderError {
    if err.is_timeout() {
        return ProviderError::Timeout;
    }
    ProviderError::Transport(err.to_string())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn http_display_strips_query_for_secret_redaction() {
        let e = ProviderError::Http {
            status: 401,
            url: "https://api.polygon.io/v2/foo?apiKey=SECRET99&x=1".to_string(),
            body_snippet: None,
        };
        let s = e.to_string();
        assert!(!s.contains("SECRET99"));
        assert!(s.contains("api.polygon.io"));
        assert!(s.contains("401"));
    }

    #[test]
    fn http_display_includes_body_snippet() {
        let e = ProviderError::Http {
            status: 403,
            url: "https://example.com/x".to_string(),
            body_snippet: Some("access denied".to_string()),
        };
        assert!(e.to_string().contains("access denied"));
    }

    #[test]
    fn http_debug_redacts_query_secrets() {
        let e = ProviderError::Http {
            status: 401,
            url: "https://api.polygon.io/v2/foo?apiKey=SECRET99&x=1".to_string(),
            body_snippet: None,
        };
        let d = format!("{e:?}");
        assert!(
            !d.contains("SECRET99"),
            "Debug leaked secret: {d}"
        );
        assert!(!d.contains("apiKey="), "Debug leaked query: {d}");
        assert!(d.contains("api.polygon.io"), "{d}");
    }

    #[test]
    fn rate_limited_display_subsecond_uses_ms() {
        let e = ProviderError::RateLimited {
            retry_after: Some(Duration::from_millis(400)),
        };
        let s = e.to_string();
        assert!(s.contains("400ms"), "{s}");
    }

    #[test]
    fn rate_limited_display_seconds_round_up_when_subsec() {
        let e = ProviderError::RateLimited {
            retry_after: Some(Duration::from_millis(1500)),
        };
        let s = e.to_string();
        assert!(
            s.contains("retry after 2s"),
            "expected ceiling seconds, got {s:?}"
        );
    }

    /// Issue #122 / SPEC §20.15.3 — `<ProviderError as Clone>::clone` lossily
    /// maps the [`ProviderError::Json`] arm to [`ProviderError::ApiMessage`].
    #[test]
    fn clone_of_json_becomes_api_message() {
        let json_err = serde_json::from_str::<u32>("not a number").unwrap_err();
        let original = ProviderError::Json(json_err);
        assert!(matches!(original, ProviderError::Json(_)));

        let cloned = original.clone();
        match cloned {
            ProviderError::ApiMessage(s) => {
                assert!(
                    s.starts_with("Invalid JSON response: "),
                    "post-clone message body: {s:?}"
                );
            }
            other => panic!("expected ApiMessage after clone, got {other:?}"),
        }
    }
}
