//! Shared [`reqwest::Client`] for all market-data HTTP (timeouts + User-Agent).

use std::fmt;
use std::sync::OnceLock;
use std::time::Duration;

/// Connect / per-request timeouts per [`docs/SPEC.md`](../../docs/SPEC.md) §19 / Issue #18.
pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper bound for [`STOCKTERM_DEBUG_HTTP_DELAY_MS`] (SPEC §38.2 / Issue #85).
pub(crate) const MAX_DEBUG_HTTP_DELAY_MS: u64 = 120_000;

/// Milliseconds for optional quote-batch delay (Issue #17 / SPEC §16.1). Parsed once; invalid → 0.
fn debug_http_delay_ms() -> u64 {
    static MS: OnceLock<u64> = OnceLock::new();
    *MS.get_or_init(|| {
        std::env::var("STOCKTERM_DEBUG_HTTP_DELAY_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    })
}

fn effective_debug_http_delay_ms() -> u64 {
    debug_http_delay_ms().min(MAX_DEBUG_HTTP_DELAY_MS)
}

/// Developer-only stall before a watchlist quote batch fan-out (default: no-op).
pub async fn maybe_debug_http_delay() {
    let ms = effective_debug_http_delay_ms();
    if ms > 0 {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

/// Browser-like prefix improves Yahoo Finance compatibility; crate identity appended per SPEC.
fn user_agent() -> String {
    format!(
        "Mozilla/5.0 (compatible; stockterm/{}; +https://github.com/FelipeMorandini/stockterm) Chrome/120.0",
        env!("CARGO_PKG_VERSION")
    )
}

static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();

/// Failure to build the shared HTTP client (TLS / proxy / builder misconfiguration).
#[derive(Debug, Clone)]
pub struct ClientInitError(String);

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to initialize HTTP client: {}. Check TLS/HTTPS certificates and proxy settings.",
            self.0
        )
    }
}

impl std::error::Error for ClientInitError {}

fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(HTTP_REQUEST_TIMEOUT)
        .connect_timeout(HTTP_CONNECT_TIMEOUT)
        .user_agent(user_agent())
        .build()
}

/// Builds the process-wide [`reqwest::Client`]. Call from `main` before any HTTP or TUI setup.
pub fn init_shared_client() -> Result<(), ClientInitError> {
    let stored = CLIENT.get_or_init(|| build_client().map_err(|e| e.to_string()));
    match stored {
        Ok(_) => Ok(()),
        Err(msg) => Err(ClientInitError(msg.clone())),
    }
}

/// Shared client for provider HTTP. [`init_shared_client`] must have succeeded first.
pub fn shared_client() -> &'static reqwest::Client {
    match CLIENT.get() {
        Some(Ok(client)) => client,
        Some(Err(_)) | None => {
            panic!("init_shared_client must run successfully before shared_client")
        }
    }
}

/// Idempotent HTTP client init for integration tests that use [`shared_client`].
#[cfg(test)]
pub fn ensure_shared_client_for_tests() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        init_shared_client().expect("shared client for tests");
    });
}

#[cfg(test)]
pub(crate) fn effective_debug_http_delay_ms_for_test(raw: u64) -> u64 {
    raw.min(MAX_DEBUG_HTTP_DELAY_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_http_delay_effective_caps_at_max() {
        assert_eq!(
            effective_debug_http_delay_ms_for_test(999_999_999),
            MAX_DEBUG_HTTP_DELAY_MS
        );
        assert_eq!(effective_debug_http_delay_ms_for_test(5000), 5000);
        assert_eq!(effective_debug_http_delay_ms_for_test(0), 0);
    }
}
