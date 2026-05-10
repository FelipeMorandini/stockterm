use crossterm::event::{self, Event as CEvent, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _tx: mpsc::Sender<Event<KeyEvent>>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);
        let event_tx = tx.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CEvent::Key(key)) => {
                            let _ = event_tx.send(Event::Input(key));
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate && event_tx.send(Event::Tick).is_ok() {
                    last_tick = Instant::now();
                }
            }
        });
        Events { rx, _tx: tx }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}