//! Structured failures from market-data HTTP layers.

use std::fmt;

#[derive(Debug)]
pub enum ProviderError {
    Timeout,
    Http {
        status: u16,
        url: String,
    },
    Json(serde_json::Error),
    ApiMessage(String),
    Transport(String),
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
            ProviderError::Http { status, url } => {
                write!(
                    f,
                    "HTTP {} ({})",
                    status,
                    url_without_query(url)
                )
            }
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
        };
        let s = e.to_string();
        assert!(!s.contains("SECRET99"));
        assert!(s.contains("api.polygon.io"));
        assert!(s.contains("401"));
    }
}
