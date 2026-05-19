//! Shared concurrency helpers for market-data fan-out (§40.2 / Issue #56).

use crate::api::ProviderError;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Acquire one quote-fetch permit, or fail closed when the semaphore is closed.
pub(crate) async fn acquire_quote_permit<'a>(
    sem: &'a Semaphore,
    symbol: &str,
    provider: &'static str,
) -> Result<SemaphorePermit<'a>, ProviderError> {
    sem.acquire().await.map_err(|_| {
        tracing::warn!(
            target: "stockterm::fetch",
            symbol,
            provider,
            "quote concurrency semaphore closed"
        );
        ProviderError::Transport("quote concurrency semaphore closed".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn acquire_quote_permit_closed_semaphore_returns_transport() {
        let sem = Arc::new(Semaphore::new(1));
        sem.close();
        let err = acquire_quote_permit(&sem, "AAPL", "test").await.unwrap_err();
        assert!(matches!(err, ProviderError::Transport(msg) if msg.contains("semaphore closed")));
    }
}
