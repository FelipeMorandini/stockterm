use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_event(app: &mut App, key: KeyEvent) {
    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.should_quit = true;
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            if c.is_ascii_uppercase() {
                app.symbol.push(c);
            }
        }
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.symbol.pop();
        }
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            tokio::spawn(app.fetch_ticker_data());
        }
        _ => {}
    }
}