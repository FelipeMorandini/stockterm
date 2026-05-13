//! Structured failures from market-data HTTP layers.

use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ProviderError {
    Timeout,
    Http {
        status: u16,
        url: String,
        body_snippet: Option<String>,
    },
    /// HTTP 429 before retries are exhausted; after exhaustion callers map to [`Http`] with status 429.
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
    Json(serde_json::Error),
    ApiMessage(String),
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

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Timeout => write!(f, "Request timed out"),
            ProviderError::Http {
                status,
                url,
                body_snippet,
            } => {
                write!(f, "HTTP {} ({})", status, url_without_query(url))?;
                if let Some(s) = body_snippet {
                    if !s.is_empty() {
                        write!(f, " — {s}")?;
                    }
                }
                Ok(())
            }
            ProviderError::RateLimited { retry_after } => match retry_after {
                Some(d) if !d.is_zero() => {
                    write!(f, "Rate limited (retry after {}s)", d.as_secs())
                }
                _ => f.write_str("Rate limited"),
            },
            ProviderError::Json(err) => write!(f, "Invalid JSON response: {err}"),
            ProviderError::ApiMessage(msg) => f.write_str(msg),
            ProviderError::Transport(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProviderError::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(value: serde_json::Error) -> Self {
        ProviderError::Json(value)
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

    /// Issue #122 / SPEC §20.15.3 — `<ProviderError as Clone>::clone` lossily
    /// maps the [`ProviderError::Json`] arm to [`ProviderError::ApiMessage`].
    /// Locks the contract documented on the `Json` variant so future code
    /// does not regress (e.g. a refactor to `Arc<serde_json::Error>` would
    /// flip this assertion and is an intentional, breaking-API change).
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
