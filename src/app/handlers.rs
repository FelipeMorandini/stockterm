use crate::app::alerts::handle_alerts_events;
use crate::app::portfolio::handle_portfolio_events;
use crate::app::{App, Tab};
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
            code: KeyCode::Tab,
            ..
        } => {
            app.next_tab();
        }
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => {
            app.prev_tab();
        }
        key => match app.active_tab {
            Tab::Portfolio => {
                handle_portfolio_events(app, key);
            }
            Tab::Alerts => {
                handle_alerts_events(app, key);
            }
            Tab::StockView => {
                handle_stock_view_keys(app, key);
            }
            _ => {}
        },
    }
}

fn handle_stock_view_keys(app: &mut App, key: KeyEvent) {
    match key {
        KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.add_current_to_watchlist();
        }
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.remove_selected_watchlist_row();
        }
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.remove_selected_watchlist_row();
        }
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.watchlist_select_next();
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.watchlist_select_prev();
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        } if c.is_ascii_uppercase() => {
            app.symbol.push(c);
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
            app.should_fetch_ticker = true;
        }
        _ => {}
    }
}
