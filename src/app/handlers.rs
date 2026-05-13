use crate::app::alerts::{cycle_alert_dialog_focus, handle_alerts_events};
use crate::app::keyboard::letter_key_plain;
use crate::app::portfolio::{cycle_portfolio_dialog_focus, handle_portfolio_events};
use crate::app::{App, SettingsEdit, Tab};
use crate::models::time_range::TimeRange;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_event(app: &mut App, key: KeyEvent) {
    // Issue #123 / SPEC §20.15.4 — plain `q` is a global quit chord, even when
    // the error log overlay is open. Handled before the overlay early-return
    // for parity with the global `Ctrl+E` / `Ctrl+R` chords below.
    if matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }
    ) {
        app.should_quit = true;
        return;
    }

    if key.code == KeyCode::Char('e') && key.modifiers == KeyModifiers::CONTROL {
        app.error_log_overlay_open = !app.error_log_overlay_open;
        // Issue #120 / SPEC §20.15.1 — clamp on open so a stale `error_log_scroll`
        // (e.g. from an earlier session before ring evictions) does not paint
        // past `max_scroll` on the first frame.
        if app.error_log_overlay_open {
            app.clamp_error_log_scroll();
        }
        return;
    }
    if key.code == KeyCode::Char('r') && key.modifiers == KeyModifiers::CONTROL {
        app.retry_last_failed_fetch();
        return;
    }

    if app.error_log_overlay_open {
        handle_error_log_overlay_keys(app, key);
        return;
    }

    match key {
        KeyEvent {
            code: KeyCode::Tab,
            ..
        } => {
            if app.active_tab == Tab::Alerts && app.alert_add_dialog.is_some() {
                cycle_alert_dialog_focus(app, true);
            } else if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
                cycle_portfolio_dialog_focus(app, true);
            } else {
                app.next_tab();
            }
        }
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => {
            if app.active_tab == Tab::Alerts && app.alert_add_dialog.is_some() {
                cycle_alert_dialog_focus(app, false);
            } else if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
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

/// Default page step for `PageUp` / `PageDown` in the error log overlay
/// (SPEC §20.15.1). At small terminal heights the *effective* step is
/// adaptively reduced so paging never overshoots the painted viewport — see
/// [`overlay_page_rows`].
const ERROR_LOG_OVERLAY_PAGE_ROWS: usize = 10;

/// Issue #120 / SPEC §20.15.1 — adaptive page step. One row of context
/// overlap (vim's `Ctrl-D`/`Ctrl-F` convention) keeps users oriented at small
/// terminal heights; clamped to the most recent layout-derived visible-row
/// count published by `draw_error_log_overlay` in `ui.rs`.
fn overlay_page_rows(app: &App) -> usize {
    let visible = app.error_log_visible_rows.max(1);
    ERROR_LOG_OVERLAY_PAGE_ROWS.min(visible.saturating_sub(1).max(1))
}

fn handle_error_log_overlay_keys(app: &mut App, key: KeyEvent) {
    // Issue #120 / #121 / SPEC §20.15.1 (round-2 audit follow-up) — re-clamp
    // on every overlay input so a recent terminal resize-larger (which shrinks
    // the layout-derived `max_scroll` but not `error_log_scroll`) does not
    // strand `k` / `PageUp` against a stale field. Without this entry-clamp,
    // the local-clamp in `draw_error_log_overlay` (Issue #121, scroll-read-only)
    // masks the staleness for *rendering* only, leaving subsequent
    // `saturating_sub`s acting on a value far above the painted bottom and
    // producing a "dead key" for many presses. Idempotent + O(1).
    app.clamp_error_log_scroll();

    match key {
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.error_log_overlay_open = false;
        }
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.error_log_scroll = app.error_log_scroll.saturating_add(1);
            app.clamp_error_log_scroll();
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.error_log_scroll = app.error_log_scroll.saturating_sub(1);
        }
        KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            let step = overlay_page_rows(app);
            app.error_log_scroll = app.error_log_scroll.saturating_add(step);
            app.clamp_error_log_scroll();
        }
        KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            let step = overlay_page_rows(app);
            app.error_log_scroll = app.error_log_scroll.saturating_sub(step);
        }
        _ => {}
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
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } if app.settings_row == 3 => {
                app.sync_settings_theme_draft_from_config();
            }
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers,
                ..
            } if app.settings_row == 3 && letter_key_plain(modifiers) => {
                app.settings_cycle_theme_draft_prev();
            }
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers,
                ..
            } if app.settings_row == 3 && letter_key_plain(modifiers) => {
                app.settings_cycle_theme_draft_next();
            }
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            } if app.settings_row == 3 => {
                app.settings_cycle_theme_draft_prev();
            }
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } if app.settings_row == 3 => {
                app.settings_cycle_theme_draft_next();
            }
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
