use crossterm::event::{self, Event as CEvent, KeyEvent};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

pub enum Event {
    Input(KeyEvent),
    Tick,
}

/// Default upper bound for waiting on the crossterm bridge thread after `event_tx` is dropped.
const DEFAULT_EVENT_JOIN_MS: u64 = 2000;

/// Bridges crossterm (blocking thread) into the async `App::run` loop.
///
/// The thread exits when [`UnboundedSender::send`] fails (receiver dropped). The caller must
/// drop the matching receiver before [`join_event_thread`].
pub fn spawn_event_thread(tx: UnboundedSender<Event>) -> JoinHandle<()> {
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
                match event::read() {
                    Ok(CEvent::Key(key)) => {
                        if tx.send(Event::Input(key)).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    })
}

fn event_join_timeout() -> Duration {
    std::env::var("STOCKTERM_EVENT_JOIN_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(DEFAULT_EVENT_JOIN_MS))
}

/// Waits for the crossterm bridge thread to exit after the async side drops the event receiver.
///
/// Times out after [`STOCKTERM_EVENT_JOIN_MS`] (default 2000 ms) and logs a warning; the helper
/// waiter thread may still be running until the bridge thread finishes.
pub fn join_event_thread(handle: JoinHandle<()>) {
    let timeout = event_join_timeout();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let _ = handle.join();
        let _ = done_tx.send(());
    });
    match done_rx.recv_timeout(timeout) {
        Ok(()) => {}
        Err(_) => {
            tracing::warn!(
                target: "stockterm::app",
                timeout_ms = timeout.as_millis(),
                "event thread join timed out"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requires a TTY for crossterm; run locally with `cargo test spawn_event_thread_exits -- --ignored`.
    #[test]
    #[ignore = "requires terminal for crossterm poll"]
    fn spawn_event_thread_exits_after_sender_drop() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = spawn_event_thread(tx);
        drop(_rx);
        join_event_thread(handle);
    }
}
