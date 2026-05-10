use crate::app::alerts::handle_alerts_events;
use crate::app::keyboard::letter_key_plain;
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
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.add_current_to_watchlist();
        }
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.remove_selected_watchlist_row();
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.eq_ignore_ascii_case(&'d')
            && modifiers.contains(KeyModifiers::SHIFT)
            && letter_key_plain(modifiers) =>
        {
            app.remove_selected_watchlist_row();
        }
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.watchlist_select_next();
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.watchlist_select_next();
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.watchlist_select_prev();
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.watchlist_select_prev();
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.is_ascii_alphabetic() && letter_key_plain(modifiers) => {
            app.symbol.push(c.to_ascii_uppercase());
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
