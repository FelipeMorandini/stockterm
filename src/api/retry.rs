//! Exponential backoff + jitter for transient HTTP failures (SPEC §19.5).

use std::time::{Duration, Instant};

use crate::api::error::ProviderError;
use crate::api::error::ProviderResult;
use crate::api::http::shared_client;
use crate::api::http_fetch::{get_text_once, FetchOnceError};

const MAX_ATTEMPTS: usize = 5;
const BASE_DELAY_MS: u64 = 500;
const BACKOFF_MULTIPLIER: u64 = 2;
const BACKOFF_CAP_MS: u64 = 30_000;

fn exp_delay_for_attempt(attempt: usize) -> Duration {
    let pow = BACKOFF_MULTIPLIER.saturating_pow(attempt as u32);
    let ms = BASE_DELAY_MS
        .saturating_mul(pow)
        .clamp(1, BACKOFF_CAP_MS);
    Duration::from_millis(ms)
}

/// ±25% jitter without adding a `rand` dependency (SPEC §19.5).
fn jitter_duration(base: Duration) -> Duration {
    let d = base.as_millis() as u64;
    if d == 0 {
        return base;
    }
    let low = d * 75 / 100;
    let high = d * 125 / 100;
    let span = (high - low).max(1);
    let tick = Instant::now().elapsed().as_nanos() as u64;
    let j = low + (tick % span);
    Duration::from_millis(j.max(1))
}

fn is_transient(err: &ProviderError) -> bool {
    match err {
        ProviderError::Timeout => true,
        ProviderError::Transport(_) => true,
        ProviderError::Http { status, .. } => (500..600).contains(status),
        ProviderError::Json(_) | ProviderError::ApiMessage(_) | ProviderError::RateLimited { .. } => {
            false
        }
    }
}

fn map_rate_limited_exhausted(url: &str, body_snippet: Option<String>) -> ProviderError {
    ProviderError::Http {
        status: 429,
        url: url.to_string(),
        body_snippet,
    }
}

pub(crate) async fn execute_get_text_with_retry(url: &str) -> ProviderResult<String> {
    execute_get_text_with_retry_inner(shared_client(), url).await
}

pub(crate) async fn execute_get_text_with_retry_inner(
    client: &reqwest::Client,
    url: &str,
) -> ProviderResult<String> {
    for attempt in 0..MAX_ATTEMPTS {
        match get_text_once(client, url).await {
            Ok(text) => return Ok(text),
            Err(FetchOnceError::RateLimited {
                retry_after,
                body_snippet,
            }) => {
                if attempt + 1 >= MAX_ATTEMPTS {
                    return Err(map_rate_limited_exhausted(url, body_snippet));
                }
                let sleep_d = match retry_after {
                    Some(d) => d,
                    None => jitter_duration(exp_delay_for_attempt(attempt)),
                };
                tokio::time::sleep(sleep_d).await;
            }
            Err(FetchOnceError::Fatal(e)) => {
                if !is_transient(&e) {
                    return Err(e);
                }
                if attempt + 1 >= MAX_ATTEMPTS {
                    return Err(e);
                }
                let sleep_d = jitter_duration(exp_delay_for_attempt(attempt));
                tokio::time::sleep(sleep_d).await;
            }
        }
    }
    // `MAX_ATTEMPTS > 0`: every iteration returns, retries, or errors out.
    unreachable!("HTTP retry loop: MAX_ATTEMPTS > 0 ensures a return inside the loop");
}

#[cfg(test)]
mod wiremock_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(crate::api::http::HTTP_CONNECT_TIMEOUT)
            .build()
            .expect("test client")
    }

    /// `Retry-After: 1` then 200 — real wall clock (reliable with `reqwest` + wiremock I/O).
    #[tokio::test]
    async fn retry_after_one_second_before_success() {
        let srv = MockServer::start().await;
        let n = Arc::new(AtomicUsize::new(0));
        Mock::given(method("GET"))
            .respond_with({
                let n = Arc::clone(&n);
                move |_req: &Request| {
                    let i = n.fetch_add(1, Ordering::SeqCst);
                    if i == 0 {
                        ResponseTemplate::new(429).append_header("retry-after", "1")
                    } else {
                        ResponseTemplate::new(200).set_body_string("ok")
                    }
                }
            })
            .expect(2)
            .mount(&srv)
            .await;

        let url = format!("{}/x", srv.uri());
        let client = test_client();
        let t0 = Instant::now();
        let out = execute_get_text_with_retry_inner(&client, &url)
            .await
            .expect("success after 429");
        assert_eq!(out, "ok");
        assert!(
            t0.elapsed() >= Duration::from_millis(900),
            "expected ~1s Retry-After wait, got {:?}",
            t0.elapsed()
        );
    }

    #[tokio::test]
    async fn five_hundred_retries_then_success() {
        let srv = MockServer::start().await;
        let n = Arc::new(AtomicUsize::new(0));
        Mock::given(method("GET"))
            .respond_with({
                let n = Arc::clone(&n);
                move |_req: &Request| {
                    let i = n.fetch_add(1, Ordering::SeqCst);
                    match i {
                        0 | 1 => ResponseTemplate::new(500),
                        _ => ResponseTemplate::new(200).set_body_string("y"),
                    }
                }
            })
            .expect(3)
            .mount(&srv)
            .await;

        let url = format!("{}/z", srv.uri());
        let client = test_client();
        let out = execute_get_text_with_retry_inner(&client, &url)
            .await
            .expect("ok");
        assert_eq!(out, "y");
    }

    #[tokio::test]
    async fn four_o_one_plain_text_not_json_primary() {
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(401)
                    .append_header("content-type", "text/plain")
                    .set_body_string("plain-no-json"),
            )
            .mount(&srv)
            .await;

        let url = format!("{}/denied", srv.uri());
        let client = test_client();
        let err = execute_get_text_with_retry_inner(&client, &url)
            .await
            .expect_err("401");

        let s = err.to_string();
        assert!(
            !s.contains("Invalid JSON response"),
            "got serde-style message: {s}"
        );
        assert!(s.contains("401") || s.contains("plain"));
    }

    #[tokio::test(start_paused = true)]
    async fn stall_triggers_timeout() {
        let srv = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(120)),
            )
            .mount(&srv)
            .await;

        let url = format!("{}/slow", srv.uri());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(200))
            .connect_timeout(Duration::from_secs(1))
            .build()
            .unwrap();

        let h = tokio::spawn(async move {
            execute_get_text_with_retry_inner(&client, &url).await
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(300)).await;

        let err = h.await.unwrap().expect_err("timeout");
        assert!(matches!(err, ProviderError::Timeout));
    }
}
