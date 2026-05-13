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
    Json(serde_json::Error),
    ApiMessage(String),
    Transport(String),
}

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
}
