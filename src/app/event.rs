use crossterm::event::{self, Event as CEvent, KeyEvent};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

pub enum Event {
    Input(KeyEvent),
    Tick,
}

/// Bridges crossterm (blocking thread) into the async `App::run` loop.
pub fn spawn_event_thread(tx: UnboundedSender<Event>) {
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
                        let _ = tx.send(Event::Input(key));
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });
}
