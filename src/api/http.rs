//! Shared [`reqwest::Client`] for all market-data HTTP (timeouts + User-Agent).

use std::sync::OnceLock;
use std::time::Duration;

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
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(user_agent())
            .build()
            .expect("reqwest Client builder")
    })
}
