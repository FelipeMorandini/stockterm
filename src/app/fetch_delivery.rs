//! Centralized `FetchDone` / `InflightRecovery` delivery (Issues #71, #78 / SPEC §39.2).

use crate::app::app::{FetchDone, InflightRecovery};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

fn warn_fetch_channel_closed(context: &str, _err: &SendError<FetchDone>) {
    tracing::warn!(
        target: "stockterm::fetch",
        context,
        "dropped fetch result (channel closed)"
    );
}

fn warn_inflight_recovery_send_failed(kind: &str, _err: SendError<InflightRecovery>) {
    tracing::warn!(
        target: "stockterm::fetch",
        kind,
        "inflight recovery send failed"
    );
}

fn fetch_done_context(done: &FetchDone) -> &'static str {
    match done {
        FetchDone::Stock { .. } => "stock quote batch result",
        FetchDone::Historical { .. } => "historical fetch result",
        FetchDone::News { .. } => "news fetch result",
        FetchDone::Search { .. } => "search fetch result",
    }
}

fn recovery_kind(recovery: InflightRecovery) -> &'static str {
    match recovery {
        InflightRecovery::Historical => "historical",
        InflightRecovery::News => "news",
        InflightRecovery::Search => "search",
        InflightRecovery::Stock => "stock",
        InflightRecovery::NewsUrlOp => "news_url_op",
    }
}

/// Delivers a fetch result to the main loop, falling back to inflight recovery on send failure.
///
/// When both sends fail, the main-loop stale-inflight watchdog clears the flag (SPEC §39.2).
pub(crate) fn deliver_fetch_done(
    fetch_tx: &UnboundedSender<FetchDone>,
    recovery_tx: Option<&UnboundedSender<InflightRecovery>>,
    done: FetchDone,
    recovery: InflightRecovery,
) {
    let context = fetch_done_context(&done);
    if let Err(e) = fetch_tx.send(done) {
        warn_fetch_channel_closed(context, &e);
        if let Some(rtx) = recovery_tx {
            if let Err(re) = rtx.send(recovery) {
                warn_inflight_recovery_send_failed(recovery_kind(recovery), re);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::app::FetchDone;
    use crate::models::time_range::TimeRange;

    #[test]
    fn deliver_fetch_done_uses_recovery_when_fetch_channel_closed() {
        let (fetch_tx, fetch_rx) = tokio::sync::mpsc::unbounded_channel();
        let (recovery_tx, mut recovery_rx) = tokio::sync::mpsc::unbounded_channel();
        drop(fetch_rx);

        deliver_fetch_done(
            &fetch_tx,
            Some(&recovery_tx),
            FetchDone::Historical {
                symbol: "AAPL".into(),
                time_range: TimeRange::M1,
                result: Err(crate::api::error::ProviderError::ApiMessage("x".into())),
            },
            InflightRecovery::Historical,
        );

        assert_eq!(
            recovery_rx.try_recv().ok(),
            Some(InflightRecovery::Historical)
        );
    }

    #[test]
    fn deliver_fetch_done_both_closed_does_not_panic() {
        let (fetch_tx, fetch_rx) = tokio::sync::mpsc::unbounded_channel();
        let (recovery_tx, recovery_rx) = tokio::sync::mpsc::unbounded_channel();
        drop(fetch_rx);
        drop(recovery_rx);

        deliver_fetch_done(
            &fetch_tx,
            Some(&recovery_tx),
            FetchDone::Search {
                generation: 1,
                query: "a".into(),
                result: Ok(crate::models::search::SymbolSearchResponse {
                    status: "OK".into(),
                    count: 0,
                    results: vec![],
                }),
            },
            InflightRecovery::Search,
        );
    }
}
