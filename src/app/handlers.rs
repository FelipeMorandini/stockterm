#![allow(clippy::collapsible_match)]

use crate::app::alerts::{cycle_alert_dialog_focus, handle_alerts_events};
use crate::app::keyboard::{letter_key_plain, tab_key_plain};
use crate::app::portfolio::{cycle_portfolio_dialog_focus, handle_portfolio_events};
use crate::app::{App, SettingsEdit, Tab};
use crate::config::keymap::{Action, BindingLayer};
use crate::models::time_range::TimeRange;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Add dialog open on Alerts or Portfolio — global Tab cycles fields instead of switching tabs (§36.2).
fn modal_add_dialog_open(app: &App) -> bool {
    (app.active_tab == Tab::Alerts && app.alert_add_dialog.is_some())
        || (app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some())
}

pub fn handle_event(app: &mut App, key: KeyEvent) {
    // Issue #123 / SPEC §20.15.4 — `Quit` is global, including when the error log overlay is open.
    if matches!(
        app.resolved_keymap.action(BindingLayer::Global, &key),
        Some(Action::Quit)
    ) {
        app.should_quit = true;
        return;
    }

    if matches!(
        app.resolved_keymap.action(BindingLayer::Global, &key),
        Some(Action::OpenErrorLog)
    ) {
        app.error_log_overlay_open = !app.error_log_overlay_open;
        if app.error_log_overlay_open {
            app.clamp_error_log_scroll();
        }
        return;
    }
    if matches!(
        app.resolved_keymap.action(BindingLayer::Global, &key),
        Some(Action::ForceRefresh)
    ) {
        app.retry_last_failed_fetch();
        return;
    }

    if app.error_log_overlay_open {
        handle_error_log_overlay_keys(app, key);
        return;
    }

    match app.resolved_keymap.action(BindingLayer::Global, &key) {
        Some(Action::GlobalTab) if tab_key_plain(key.modifiers) => {
            if app.active_tab == Tab::Alerts && app.alert_add_dialog.is_some() {
                cycle_alert_dialog_focus(app, true);
            } else if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
                cycle_portfolio_dialog_focus(app, true);
            } else {
                app.next_tab();
            }
        }
        Some(Action::GlobalTab)
            if !tab_key_plain(key.modifiers) && modal_add_dialog_open(app) => {}
        Some(Action::GlobalBackTab) if tab_key_plain(key.modifiers) => {
            if app.active_tab == Tab::Alerts && app.alert_add_dialog.is_some() {
                cycle_alert_dialog_focus(app, false);
            } else if app.active_tab == Tab::Portfolio && app.portfolio_dialog.is_some() {
                cycle_portfolio_dialog_focus(app, false);
            } else {
                app.prev_tab();
            }
        }
        Some(Action::GlobalBackTab)
            if !tab_key_plain(key.modifiers) && modal_add_dialog_open(app) => {}
        _ => match app.active_tab {
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

/// Default page step for `PageUp` / `PageDown` in the error log overlay (SPEC §20.15.1).
const ERROR_LOG_OVERLAY_PAGE_ROWS: usize = 10;

fn overlay_page_rows(app: &App) -> usize {
    let visible = app.error_log_visible_rows.max(1);
    ERROR_LOG_OVERLAY_PAGE_ROWS.min(visible.saturating_sub(1).max(1))
}

fn handle_error_log_overlay_keys(app: &mut App, key: KeyEvent) {
    app.clamp_error_log_scroll();

    match app.resolved_keymap.action(BindingLayer::ErrorOverlay, &key) {
        Some(Action::OverlayClose) => {
            app.error_log_overlay_open = false;
        }
        Some(Action::OverlayScrollDown) => {
            app.error_log_scroll = app.error_log_scroll.saturating_add(1);
            app.clamp_error_log_scroll();
        }
        Some(Action::OverlayScrollUp) => {
            app.error_log_scroll = app.error_log_scroll.saturating_sub(1);
        }
        Some(Action::OverlayPageDown) => {
            let step = overlay_page_rows(app);
            app.error_log_scroll = app.error_log_scroll.saturating_add(step);
            app.clamp_error_log_scroll();
        }
        Some(Action::OverlayPageUp) => {
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
    use Action::*;
    if let Some(a) = app.resolved_keymap.action(BindingLayer::Charts, &key) {
        match a {
            ChartRangeD1 => app.set_charts_time_range(TimeRange::D1),
            ChartRangeW1 => app.set_charts_time_range(TimeRange::W1),
            ChartRangeM1 => app.set_charts_time_range(TimeRange::M1),
            ChartRangeY1 => app.set_charts_time_range(TimeRange::Y1),
            ChartResetViewport => app.charts_reset_viewport(),
            ChartZoomIn | ChartZoomOut => {
                if charts_zoom_modifiers_ok(key.modifiers) {
                    match a {
                        ChartZoomIn => app.charts_zoom_in(),
                        ChartZoomOut => app.charts_zoom_out(),
                        _ => {}
                    }
                }
            }
            ChartPanLeft => {
                if key.code == KeyCode::Left || letter_key_plain(key.modifiers) {
                    app.charts_pan_left();
                }
            }
            ChartPanRight => {
                if key.code == KeyCode::Right || letter_key_plain(key.modifiers) {
                    app.charts_pan_right();
                }
            }
            ChartToggleCandle => {
                if letter_key_plain(key.modifiers) {
                    app.charts_toggle_mode();
                }
            }
            _ => {}
        }
    }
}

fn handle_search_events(app: &mut App, key: KeyEvent) {
    // §26 / Issue #136: `SearchEsc`, `SearchEnter`, and `SearchBackspace` intentionally require
    // `KeyModifiers::NONE` so Ctrl/Alt chords do not clear the query or pick a row by accident.
    use Action::*;
    if let Some(a) = app.resolved_keymap.action(BindingLayer::Search, &key) {
        match a {
            SearchEsc if key.modifiers == KeyModifiers::NONE => {
                // Issue #60 / SPEC §33 — domain-gated error clear inside `search_esc_reset`.
                app.search_esc_reset();
            }
            SearchBackspace if key.modifiers == KeyModifiers::NONE => {
                app.search_query.pop();
                if app.search_query.trim().is_empty() {
                    app.search_results = None;
                    app.search_table_state.select(None);
                }
                app.touch_search_debounce();
            }
            SearchEnter if key.modifiers == KeyModifiers::NONE => {
                app.search_pick_symbol_go_stock();
            }
            SearchRowDown => {
                if key.code == KeyCode::Down || letter_key_plain(key.modifiers) {
                    app.search_select_next();
                }
            }
            SearchRowUp => {
                if key.code == KeyCode::Up || letter_key_plain(key.modifiers) {
                    app.search_select_prev();
                }
            }
            _ => {}
        }
        return;
    }

    if let KeyEvent {
        code: KeyCode::Char(c),
        modifiers,
        ..
    } = key
    {
        if search_query_char(c) && letter_key_plain(modifiers) {
            let c = if c.is_ascii_alphabetic() {
                c.to_ascii_uppercase()
            } else {
                c
            };
            app.search_query.push(c);
            app.touch_search_debounce();
        }
    }
}

fn search_query_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '.'
}

fn handle_news_events(app: &mut App, key: KeyEvent) {
    use Action::*;
    if let Some(a) = app.resolved_keymap.action(BindingLayer::News, &key) {
        match a {
            NewsRowDown => {
                if key.code == KeyCode::Down || letter_key_plain(key.modifiers) {
                    app.news_select_next();
                }
            }
            NewsRowUp => {
                if key.code == KeyCode::Up || letter_key_plain(key.modifiers) {
                    app.news_select_prev();
                }
            }
            NewsEnter if key.modifiers == KeyModifiers::NONE => {
                app.news_try_open_selected();
            }
            NewsCopyUrl if key.modifiers == KeyModifiers::NONE => {
                app.news_try_copy_selected();
            }
            _ => {}
        }
    }
}

/// Shared Esc / Enter / Backspace for Settings edit rows (Issue #136 / SPEC §26).
fn settings_edit_apply_common_action(app: &mut App, key: &KeyEvent, action: Action) -> bool {
    use Action::*;
    match action {
        SettingsEditEsc if key.modifiers == KeyModifiers::NONE => {
            app.settings_cancel_edit();
            true
        }
        SettingsEditEnter if key.modifiers == KeyModifiers::NONE => {
            let _ = app.settings_commit_edit();
            true
        }
        SettingsEditBackspace if key.modifiers == KeyModifiers::NONE => {
            app.settings_edit_buffer.pop();
            true
        }
        _ => false,
    }
}

fn settings_edit_append_digit(app: &mut App, key: &KeyEvent) -> bool {
    if !letter_key_plain(key.modifiers) {
        return false;
    }
    let KeyCode::Char(c) = key.code else {
        return false;
    };
    if !c.is_ascii_digit() {
        return false;
    }
    app.settings_edit_buffer.push(c);
    true
}

fn settings_edit_append_symbol_char(app: &mut App, key: &KeyEvent) -> bool {
    if !letter_key_plain(key.modifiers) {
        return false;
    }
    let KeyCode::Char(c) = key.code else {
        return false;
    };
    if !(c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return false;
    }
    let c = if c.is_ascii_alphabetic() {
        c.to_ascii_uppercase()
    } else {
        c
    };
    app.settings_edit_buffer.push(c);
    true
}

fn settings_edit_apply_keymap_action(app: &mut App, key: &KeyEvent, action: Action, mode: SettingsEdit) {
    use Action::*;
    if settings_edit_apply_common_action(app, key, action) {
        return;
    }
    match (action, mode) {
        (SettingsEditDigit, _) => {
            let _ = settings_edit_append_digit(app, key);
        }
        (SettingsEditSymbolChar, SettingsEdit::DefaultSymbol) => {
            let _ = settings_edit_append_symbol_char(app, key);
        }
        // Refresh-rate row: `char:a`–`z` chords resolve to `SettingsEditSymbolChar` but letters are N/A.
        (SettingsEditSymbolChar, SettingsEdit::RefreshRate) => {}
        _ => {}
    }
}

/// Wildcard append when no `SettingsEdit` chord matched (e.g. Shift+letter).
fn settings_edit_apply_unmatched_wildcard(app: &mut App, key: &KeyEvent, mode: SettingsEdit) {
    match mode {
        SettingsEdit::RefreshRate => {
            let _ = settings_edit_append_digit(app, key);
        }
        SettingsEdit::DefaultSymbol => {
            let _ = settings_edit_append_symbol_char(app, key);
        }
    }
}

fn handle_settings_events(app: &mut App, key: KeyEvent) {
    use Action::*;
    if let Some(mode) = app.settings_editing {
        if let Some(a) = app
            .resolved_keymap
            .action(BindingLayer::SettingsEdit, &key)
        {
            settings_edit_apply_keymap_action(app, &key, a, mode);
        } else {
            settings_edit_apply_unmatched_wildcard(app, &key, mode);
        }
        return;
    }

    if let Some(a) = app
        .resolved_keymap
        .action(BindingLayer::SettingsBrowse, &key)
    {
        match a {
            SettingsEscThemeDraft if key.modifiers == KeyModifiers::NONE => {
                if app.settings_row == 3 {
                    app.sync_settings_theme_draft_from_config();
                }
                if app.settings_row == 6 {
                    app.sync_settings_layout_draft_from_config();
                }
            }
            SettingsThemePrev => {
                if app.settings_row == 3
                    && (key.code == KeyCode::Left || letter_key_plain(key.modifiers))
                {
                    app.settings_cycle_theme_draft_prev();
                } else if app.settings_row == 6
                    && (key.code == KeyCode::Left || letter_key_plain(key.modifiers))
                {
                    app.settings_cycle_layout_draft_prev();
                }
            }
            SettingsThemeNext => {
                if app.settings_row == 3
                    && (key.code == KeyCode::Right || letter_key_plain(key.modifiers))
                {
                    app.settings_cycle_theme_draft_next();
                } else if app.settings_row == 6
                    && (key.code == KeyCode::Right || letter_key_plain(key.modifiers))
                {
                    app.settings_cycle_layout_draft_next();
                }
            }
            SettingsRowDown => {
                if key.code == KeyCode::Down || letter_key_plain(key.modifiers) {
                    app.settings_row_next();
                }
            }
            SettingsRowUp => {
                if key.code == KeyCode::Up || letter_key_plain(key.modifiers) {
                    app.settings_row_prev();
                }
            }
            SettingsEnter if key.modifiers == KeyModifiers::NONE => {
                app.settings_try_enter_row();
            }
            _ => {}
        }
    }
}

fn handle_stock_view_keys(app: &mut App, key: KeyEvent) {
    if app.consume_filter_input_key(&key) {
        return;
    }

    use Action::*;
    if let Some(a) = app.resolved_keymap.action(BindingLayer::StockView, &key) {
        match a {
            StockFilterToggle if key.modifiers == KeyModifiers::NONE => {
                app.filter_input_mode = true;
            }
            WatchlistAdd if letter_key_plain(key.modifiers) && !app.filter_input_mode => {
                app.add_current_to_watchlist();
            }
            WatchlistRemove if letter_key_plain(key.modifiers) && !app.filter_input_mode => {
                app.remove_selected_watchlist_row();
            }
            WatchlistRemoveShift => {
                if letter_key_plain(key.modifiers) && !app.filter_input_mode {
                    app.remove_selected_watchlist_row();
                }
            }
            StockRowDown => {
                if !app.filter_input_mode
                    && (key.code == KeyCode::Down || letter_key_plain(key.modifiers))
                {
                    app.watchlist_select_next();
                }
            }
            StockRowUp => {
                if !app.filter_input_mode
                    && (key.code == KeyCode::Up || letter_key_plain(key.modifiers))
                {
                    app.watchlist_select_prev();
                }
            }
            StockBackspace if key.modifiers == KeyModifiers::NONE && !app.filter_input_mode => {
                app.symbol.pop();
            }
            StockEnter if key.modifiers == KeyModifiers::NONE && !app.filter_input_mode => {
                app.should_fetch_ticker = true;
            }
            _ => {}
        }
        return;
    }

    // §24.5 / §26 — Stock symbol buffer: letters only (wildcard after keymap); digits are not ticker input here.
    if let KeyEvent {
        code: KeyCode::Char(c),
        modifiers,
        ..
    } = key
    {
        if c.is_ascii_alphabetic() && letter_key_plain(modifiers) && !app.filter_input_mode {
            app.symbol.push(c.to_ascii_uppercase());
        }
    }
}
