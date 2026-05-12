//! Shared [`reqwest::Client`] for all market-data HTTP (timeouts + User-Agent).

use std::sync::OnceLock;
use std::time::Duration;

/// Connect / per-request timeouts per [`docs/SPEC.md`](../../docs/SPEC.md) §19 / Issue #18.
pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

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

/// Developer-only stall before a watchlist quote batch fan-out (default: no-op).
pub async fn maybe_debug_http_delay() {
    let ms = debug_http_delay_ms();
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

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn shared_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(HTTP_REQUEST_TIMEOUT)
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .user_agent(user_agent())
            .build()
            .expect("reqwest Client builder")
    })
}
