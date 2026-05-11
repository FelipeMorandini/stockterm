use crate::app::alerts::handle_alerts_events;
use crate::app::keyboard::letter_key_plain;
use crate::app::portfolio::{cycle_portfolio_dialog_focus, handle_portfolio_events};
use crate::app::{App, SettingsEdit, Tab};
use crate::models::time_range::TimeRange;
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
            if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
                cycle_portfolio_dialog_focus(app, true);
            } else {
                app.next_tab();
            }
        }
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => {
            if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
                cycle_portfolio_dialog_focus(app, false);
            } else {
                app.prev_tab();
            }
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
            Tab::Search => {
                handle_search_events(app, key);
            }
            Tab::News => {
                handle_news_events(app, key);
            }
            Tab::Settings => {
                handle_settings_events(app, key);
            }
            Tab::Charts => {
                handle_charts_events(app, key);
            }
        },
    }
}

fn charts_zoom_modifiers_ok(m: KeyModifiers) -> bool {
    !m.intersects(
        KeyModifiers::CONTROL
            | KeyModifiers::ALT
            | KeyModifiers::META
            | KeyModifiers::SUPER
            | KeyModifiers::HYPER,
    )
}

fn handle_charts_events(app: &mut App, key: KeyEvent) {
    match key {
        KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::NONE,
            ..
        } => app.set_charts_time_range(TimeRange::D1),
        KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            ..
        } => app.set_charts_time_range(TimeRange::W1),
        KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
            ..
        } => app.set_charts_time_range(TimeRange::M1),
        KeyEvent {
            code: KeyCode::Char('4'),
            modifiers: KeyModifiers::NONE,
            ..
        } => app.set_charts_time_range(TimeRange::Y1),
        KeyEvent {
            code: KeyCode::Char('0'),
            modifiers: KeyModifiers::NONE,
            ..
        } => app.charts_reset_viewport(),
        KeyEvent {
            code: KeyCode::Char('+'),
            modifiers,
            ..
        } if charts_zoom_modifiers_ok(modifiers) => app.charts_zoom_in(),
        KeyEvent {
            code: KeyCode::Char('='),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::SHIFT) && charts_zoom_modifiers_ok(modifiers) => {
            app.charts_zoom_in();
        }
        KeyEvent {
            code: KeyCode::Char('-'),
            modifiers,
            ..
        } if charts_zoom_modifiers_ok(modifiers) => app.charts_zoom_out(),
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => app.charts_pan_left(),
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => app.charts_pan_right(),
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => app.charts_pan_left(),
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => app.charts_pan_right(),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => app.charts_toggle_mode(),
        _ => {}
    }
}

fn handle_search_events(app: &mut App, key: KeyEvent) {
    match key {
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.search_esc_reset();
        }
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.search_query.pop();
            if app.search_query.trim().is_empty() {
                app.search_results = None;
                app.search_table_state.select(None);
            }
            app.touch_search_debounce();
        }
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.search_pick_symbol_go_stock();
        }
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.search_select_next();
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.search_select_next();
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.search_select_prev();
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.search_select_prev();
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if search_query_char(c) && letter_key_plain(modifiers) => {
            let c = if c.is_ascii_alphabetic() {
                c.to_ascii_uppercase()
            } else {
                c
            };
            app.search_query.push(c);
            app.touch_search_debounce();
        }
        _ => {}
    }
}

fn search_query_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '.'
}

fn handle_news_events(app: &mut App, key: KeyEvent) {
    match key {
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.news_select_next();
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.news_select_next();
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers,
            ..
        } if letter_key_plain(modifiers) => {
            app.news_select_prev();
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.news_select_prev();
        }
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.news_try_open_selected();
        }
        _ => {}
    }
}

fn handle_settings_events(app: &mut App, key: KeyEvent) {
    match app.settings_editing {
        Some(SettingsEdit::RefreshRate) => match key {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                app.settings_cancel_edit();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                let _ = app.settings_commit_edit();
            }
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                app.settings_edit_buffer.pop();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                ..
            } if c.is_ascii_digit() && letter_key_plain(modifiers) => {
                app.settings_edit_buffer.push(c);
            }
            _ => {}
        },
        Some(SettingsEdit::DefaultSymbol) => match key {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                app.settings_cancel_edit();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                let _ = app.settings_commit_edit();
            }
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                app.settings_edit_buffer.pop();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                ..
            } if (c.is_ascii_alphanumeric() || c == '.' || c == '-')
                && letter_key_plain(modifiers) =>
            {
                let c = if c.is_ascii_alphabetic() {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                app.settings_edit_buffer.push(c);
            }
            _ => {}
        },
        None => match key {
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers,
                ..
            } if letter_key_plain(modifiers) => {
                app.settings_row_next();
            }
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                app.settings_row_next();
            }
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers,
                ..
            } if letter_key_plain(modifiers) => {
                app.settings_row_prev();
            }
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => {
                app.settings_row_prev();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                app.settings_try_enter_row();
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
