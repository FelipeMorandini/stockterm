use crate::app::styles::ResolvedTheme;
use crate::app::app_error::{AppError, ErrorSourceDomain};
use crate::app::keyboard::letter_key_plain;
use crate::app::layout::centered_rect;
use crate::app::{AlertAddDialog, AlertAddField, App, Tab};
use crate::models::alerts::{process_alert_crossings, Alert, AlertCondition};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use std::io::{self, Write};

/// Prefix for `AppError::ConfigSave` / status line when `Config::try_save` fails in `save_alerts` (§18.14.2).
pub(crate) const ALERTS_SAVE_ERROR_PREFIX: &str = "Failed to save alerts:";

const MAX_ALERT_FIELD_LEN: usize = 24;

#[cfg(any(test, feature = "desktop-notify"))]
const NOTIFY_SYMBOL_DISPLAY_MAX_CHARS: usize = 32;

/// Max UTF-8 byte length for the assembled desktop notification `body` (§18.15.3 / Issue #104).
#[cfg(any(test, feature = "desktop-notify"))]
const NOTIFY_BATCH_BODY_MAX_BYTES: usize = 1024;

/// If `s` exceeds `max_bytes` UTF-8, truncate at a char boundary and append `…` so the result
/// is at most `max_bytes` bytes total.
#[cfg(any(test, feature = "desktop-notify"))]
fn truncate_utf8_notify_body_to_max_bytes(s: &str, max_bytes: usize) -> String {
    const ELLIPSIS: &str = "…";
    let el = ELLIPSIS.len();
    if s.len() <= max_bytes {
        return s.to_string();
    }
    if max_bytes < el {
        return String::new();
    }
    let prefix_max = max_bytes - el;
    let mut end = 0usize;
    for c in s.chars() {
        let cl = c.len_utf8();
        if end + cl > prefix_max {
            break;
        }
        end += cl;
    }
    format!("{}{}", &s[..end], ELLIPSIS)
}

/// Strip control characters, collapse whitespace, cap length for OS notification bodies (§18.14.4).
#[cfg(any(test, feature = "desktop-notify"))]
pub(crate) fn sanitize_alert_notify_display_text(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let collapsed = mapped.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();
    if trimmed.chars().count() > NOTIFY_SYMBOL_DISPLAY_MAX_CHARS {
        format!(
            "{}…",
            trimmed.chars().take(NOTIFY_SYMBOL_DISPLAY_MAX_CHARS).collect::<String>()
        )
    } else {
        trimmed.to_string()
    }
}

fn ring_terminal_bell() {
    let mut out = io::stdout();
    let _ = out.write_all(b"\x07");
    let _ = out.flush();
}

pub(crate) fn alerts_tab_banner_active(app: &App) -> bool {
    app.alerts_save_retry_pending
        || app
            .error_message()
            .as_deref()
            .is_some_and(|m| m.contains(ALERTS_SAVE_ERROR_PREFIX))
}

#[cfg(feature = "desktop-notify")]
fn spawn_desktop_alert_notifications_batch(summary: String, body_lines: Vec<String>) {
    std::thread::spawn(move || {
        let joined = body_lines.join("\n");
        let body = truncate_utf8_notify_body_to_max_bytes(&joined, NOTIFY_BATCH_BODY_MAX_BYTES);
        let show_result = notify_rust::Notification::new()
            .summary(&summary)
            .body(&body)
            .show();
        if matches!(
            std::env::var("STOCKTERM_DEBUG_ALERT_NOTIFY"),
            Ok(ref s) if s == "1"
        ) {
            eprintln!("stockterm: Notification::show() = {show_result:?}");
        }
    });
}

pub fn draw_alerts(f: &mut Frame, app: &mut App, area: Rect, theme: ResolvedTheme) {
    let border_st = Style::default().fg(theme.border).bg(theme.background);
    let show_banner = alerts_tab_banner_active(app);
    let chunks = if show_banner {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area)
    };
    let main = if show_banner { chunks[1] } else { chunks[0] };

    if show_banner {
        let banner = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Alert state may not be saved to disk (TRIGGERED may be memory-only).",
                theme.fg_border(),
            )]),
            Line::from(vec![Span::styled(
                "Fix path/permissions/quota; save retries on the next quote batch.",
                theme.fg_border(),
            )]),
        ])
        .style(theme.canvas())
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(theme.canvas())
                .border_style(border_st),
        );
        f.render_widget(banner, chunks[0]);
    }

    if app.alerts.is_empty() && app.alert_add_dialog.is_none() {
        let block = Block::default()
            .title("Price Alerts")
            .borders(Borders::ALL)
            .style(theme.canvas())
            .border_style(border_st);
        let no_data_text = Line::from(vec![Span::styled(
            "No alerts configured. Add with a or A (Shift+a works).",
            theme.fg_border(),
        )]);
        let paragraph = Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, main);
        return;
    }

    if app.alerts.is_empty() {
        let block = Block::default()
            .title("Price Alerts")
            .borders(Borders::ALL)
            .style(theme.canvas())
            .border_style(border_st);
        let no_data_text = Line::from(vec![Span::styled(
            "No alerts configured. Add with a or A (Shift+a works).",
            theme.fg_border(),
        )]);
        let paragraph = Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, main);
    } else {
        let selected_style = Style::default()
            .bg(theme.selection)
            .fg(theme.foreground)
            .add_modifier(Modifier::BOLD);

        let header_cells = ["Symbol", "Condition", "Price", "Current", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.fg_foreground()));

        let header = Row::new(header_cells)
            .style(theme.canvas().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = app.alerts.iter().map(|alert| {
            let current_opt = app.get_current_price(&alert.symbol);
            let current_cell = current_opt
                .map(|p| format!("${p:.2}"))
                .unwrap_or_else(|| "—".to_string());

            let condition_text = match alert.condition {
                AlertCondition::Above => "Above",
                AlertCondition::Below => "Below",
            };

            let (status_text, status_color) = if alert.triggered {
                ("TRIGGERED", theme.negative)
            } else if current_opt.is_some() {
                ("Armed", theme.border)
            } else {
                ("No quote", theme.muted)
            };

            let cells = [
                Cell::from(alert.symbol.clone()),
                Cell::from(condition_text),
                Cell::from(format!("${:.2}", alert.price)),
                Cell::from(current_cell),
                Cell::from(status_text).style(theme.fg_color(status_color)),
            ];

            Row::new(cells).height(1).style(theme.canvas())
        });

        let table = Table::new(
            rows,
            [
                Constraint::Min(6),
                Constraint::Length(10),
                Constraint::Length(11),
                Constraint::Length(11),
                Constraint::Min(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Price Alerts")
                .style(theme.canvas())
                .border_style(border_st),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");

        f.render_stateful_widget(table, main, &mut app.alerts_state);
    }

    if app.alert_add_dialog.is_some() {
        draw_alert_add_overlay(f, app, area, theme);
    }
}

fn draw_alert_add_overlay(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let Some(dialog) = app.alert_add_dialog.as_ref() else {
        return;
    };

    f.render_widget(Clear, area);

    let popup = centered_rect(area, 55, 42);
    let border_st = Style::default().fg(theme.border).bg(theme.background);

    let sym_style = if dialog.focused == AlertAddField::Symbol {
        theme.fg_accent()
    } else {
        theme.fg_foreground()
    };
    let cond_style = if dialog.focused == AlertAddField::Condition {
        theme.fg_accent()
    } else {
        theme.fg_foreground()
    };
    let thr_style = if dialog.focused == AlertAddField::Threshold {
        theme.fg_accent()
    } else {
        theme.fg_foreground()
    };

    let cond_label = match dialog.condition {
        AlertCondition::Above => "Above",
        AlertCondition::Below => "Below",
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(
            "Add price alert — Esc cancel · Tab / Shift+Tab or ; cycle field · ←/→ on Condition (Below/Above) · Enter advances / saves on Threshold",
            theme.canvas(),
        )]),
        Line::from(vec![
            Span::styled("Symbol:    ", sym_style),
            Span::styled(dialog.symbol_buffer.as_str(), theme.fg_foreground()),
        ]),
        Line::from(vec![
            Span::styled("Condition: ", cond_style),
            Span::styled(cond_label, theme.fg_foreground()),
            Span::styled("  (; toggles · ← Below · → Above · a/A · b/B)", theme.fg_muted()),
        ]),
        Line::from(vec![
            Span::styled("Threshold: ", thr_style),
            Span::styled("$", theme.fg_foreground()),
            Span::styled(dialog.threshold_buffer.as_str(), theme.fg_foreground()),
        ]),
    ];

    if let Some(ref err) = dialog.inline_error {
        lines.push(Line::from(vec![Span::styled(
            err.as_str(),
            theme.error_text(),
        )]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add alert")
        .style(theme.canvas())
        .border_style(border_st);
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, popup);
}

pub(crate) fn cycle_alert_dialog_focus(app: &mut App, forward: bool) {
    let Some(d) = app.alert_add_dialog.as_mut() else {
        return;
    };
    use AlertAddField::*;
    d.focused = match (d.focused, forward) {
        (Symbol, true) | (Threshold, false) => Condition,
        (Condition, true) | (Symbol, false) => Threshold,
        (Threshold, true) | (Condition, false) => Symbol,
    };
    d.inline_error = None;
}

fn parse_alert_threshold(s: &str) -> Result<f64, &'static str> {
    let t = s.trim();
    if t.is_empty() {
        return Err("Threshold is required.");
    }
    let v: f64 = t.parse().map_err(|_| "Invalid threshold.")?;
    if !v.is_finite() || v <= 0.0 {
        return Err("Threshold must be a positive number.");
    }
    Ok(v)
}

fn try_commit_alert_dialog(app: &mut App) {
    let Some(ref dlg) = app.alert_add_dialog else {
        return;
    };
    let Some(sym) = crate::app::normalize_symbol(&dlg.symbol_buffer) else {
        if let Some(d) = app.alert_add_dialog.as_mut() {
            d.inline_error = Some("Symbol cannot be empty.".into());
        }
        return;
    };
    match parse_alert_threshold(&dlg.threshold_buffer) {
        Ok(price) => {
            let condition = dlg.condition;
            app.alert_add_dialog = None;
            app.add_alert(sym, condition, price);
        }
        Err(e) => {
            if let Some(d) = app.alert_add_dialog.as_mut() {
                d.inline_error = Some(e.into());
            }
        }
    }
}

fn append_symbol_char(buf: &mut String, c: char) -> bool {
    if buf.len() >= MAX_ALERT_FIELD_LEN {
        return false;
    }
    if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
        buf.push(if c.is_ascii_alphabetic() {
            c.to_ascii_uppercase()
        } else {
            c
        });
        return true;
    }
    false
}

fn append_threshold_char(buf: &mut String, c: char) -> bool {
    if buf.len() >= MAX_ALERT_FIELD_LEN {
        return false;
    }
    if c.is_ascii_digit() {
        buf.push(c);
        return true;
    }
    if c == '.' && !buf.contains('.') {
        buf.push('.');
        return true;
    }
    false
}

fn handle_alert_dialog_keys(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    match key.code {
        Esc if key.modifiers == KeyModifiers::NONE => {
            app.alert_add_dialog = None;
        }
        Tab => {
            cycle_alert_dialog_focus(app, true);
        }
        BackTab => {
            cycle_alert_dialog_focus(app, false);
        }
        Left if key.modifiers == KeyModifiers::NONE => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            if d.focused == AlertAddField::Condition {
                d.condition = AlertCondition::Below;
                d.inline_error = None;
            }
        }
        Right if key.modifiers == KeyModifiers::NONE => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            if d.focused == AlertAddField::Condition {
                d.condition = AlertCondition::Above;
                d.inline_error = None;
            }
        }
        Char(';') if letter_key_plain(key.modifiers) => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            if d.focused == AlertAddField::Condition {
                d.condition = match d.condition {
                    AlertCondition::Above => AlertCondition::Below,
                    AlertCondition::Below => AlertCondition::Above,
                };
                d.inline_error = None;
            } else {
                cycle_alert_dialog_focus(app, true);
            }
        }
        Enter if key.modifiers == KeyModifiers::NONE => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            d.inline_error = None;
            match d.focused {
                AlertAddField::Symbol => d.focused = AlertAddField::Condition,
                AlertAddField::Condition => d.focused = AlertAddField::Threshold,
                AlertAddField::Threshold => try_commit_alert_dialog(app),
            }
        }
        Backspace if key.modifiers == KeyModifiers::NONE => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            d.inline_error = None;
            match d.focused {
                AlertAddField::Symbol => {
                    d.symbol_buffer.pop();
                }
                AlertAddField::Threshold => {
                    d.threshold_buffer.pop();
                }
                AlertAddField::Condition => {}
            }
        }
        Char(c) if letter_key_plain(key.modifiers) => {
            let Some(d) = app.alert_add_dialog.as_mut() else {
                return;
            };
            d.inline_error = None;
            match d.focused {
                AlertAddField::Symbol => {
                    let _ = append_symbol_char(&mut d.symbol_buffer, c);
                }
                AlertAddField::Threshold => {
                    let _ = append_threshold_char(&mut d.threshold_buffer, c);
                }
                AlertAddField::Condition => {
                    if c.eq_ignore_ascii_case(&'a') {
                        d.condition = AlertCondition::Above;
                    } else if c.eq_ignore_ascii_case(&'b') {
                        d.condition = AlertCondition::Below;
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn handle_alerts_events(app: &mut App, key: KeyEvent) {
    if app.alert_add_dialog.is_some() {
        handle_alert_dialog_keys(app, key);
        return;
    }

    match key {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.eq_ignore_ascii_case(&'a') && letter_key_plain(modifiers) => {
            app.alert_add_dialog = Some(AlertAddDialog::new_from_app(app));
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.eq_ignore_ascii_case(&'d') && letter_key_plain(modifiers) => {
            if let Some(selected) = app.alerts_state.selected() {
                app.remove_alert(selected);
            }
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            if app.alerts.is_empty() {
                return;
            }
            match app.alerts_state.selected() {
                None => app
                    .alerts_state
                    .select(Some(app.alerts.len().saturating_sub(1))),
                Some(i) if i > 0 => app.alerts_state.select(Some(i - 1)),
                _ => {}
            }
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            if app.alerts.is_empty() {
                return;
            }
            match app.alerts_state.selected() {
                None => app.alerts_state.select(Some(0)),
                Some(i) if i < app.alerts.len().saturating_sub(1) => {
                    app.alerts_state.select(Some(i + 1));
                }
                _ => {}
            }
        }
        _ => {}
    }
}

impl App {
    pub fn add_alert(&mut self, symbol: String, condition: AlertCondition, price: f64) {
        self.alerts.push(Alert {
            symbol,
            condition,
            price,
            triggered: false,
        });

        self.save_alerts();

        if !self.alerts.is_empty() && self.alerts_state.selected().is_none() {
            self.alerts_state.select(Some(self.alerts.len() - 1));
        }
    }

    pub fn remove_alert(&mut self, index: usize) {
        if index >= self.alerts.len() {
            return;
        }

        self.alerts.remove(index);
        self.save_alerts();

        if self.alerts.is_empty() {
            self.alerts_state.select(None);
        } else if let Some(mut sel) = self.alerts_state.selected() {
            if index < sel {
                sel -= 1;
            } else if index == sel {
                sel = sel.min(self.alerts.len() - 1);
            }
            self.alerts_state.select(Some(sel));
        }
    }

    pub fn check_alerts(&mut self) {
        let prices: Vec<(String, f64)> = self
            .alerts
            .iter()
            .filter_map(|alert| {
                self.get_current_price(&alert.symbol)
                    .map(|price| (alert.symbol.clone(), price))
            })
            .collect();

        let newly = process_alert_crossings(&mut self.alerts, &prices);
        if newly.is_empty() {
            return;
        }

        for _ in &newly {
            ring_terminal_bell();
        }

        if self.config.notifications_enabled {
            #[cfg(feature = "desktop-notify")]
            {
                const K: usize = 5;
                let mut body_lines: Vec<String> = Vec::with_capacity(newly.len().min(K) + 1);
                for idx in &newly {
                    let alert = &self.alerts[*idx];
                    let last = prices
                        .iter()
                        .find(|(s, _)| s == &alert.symbol)
                        .map(|(_, p)| *p);
                    let sym = sanitize_alert_notify_display_text(&alert.symbol);
                    let cond_s = match alert.condition {
                        AlertCondition::Above => "Above",
                        AlertCondition::Below => "Below",
                    };
                    let mut line = format!("{sym} {cond_s} ${:.2}", alert.price);
                    if let Some(p) = last {
                        line.push_str(&format!(" · last ${p:.2}"));
                    }
                    body_lines.push(line);
                }
                let len = body_lines.len();
                let (summary, lines_for_notify) = if len == 1 {
                    ("StockTerm".to_string(), body_lines)
                } else {
                    let summary = format!("StockTerm — {len} alerts");
                    let mut body: Vec<String> = body_lines.iter().take(K).cloned().collect();
                    if len > K {
                        body.push(format!("… and {} more", len - K));
                    }
                    (summary, body)
                };
                spawn_desktop_alert_notifications_batch(summary, lines_for_notify);
            }
        }

        self.save_alerts();
    }

    pub(crate) fn retry_alerts_save_if_pending(&mut self) {
        if self.alerts_save_retry_pending {
            self.save_alerts();
        }
    }

    fn save_alerts(&mut self) {
        self.config.alerts = self.alerts.clone();
        match self.config.try_save() {
            Ok(()) => {
                self.alerts_save_retry_pending = false;
                self.clear_alerts_save_runtime_error_after_recovery();
            }
            Err(e) => {
                self.alerts_save_retry_pending = true;
                self.surface_runtime_error(
                    Tab::Alerts,
                    ErrorSourceDomain::Alerts,
                    AppError::ConfigSave(format!("{ALERTS_SAVE_ERROR_PREFIX} {e}")),
                    true,
                );
            }
        }
    }

    pub fn get_current_price(&self, symbol: &str) -> Option<f64> {
        if let Some(ticker_data) = &self.ticker_data {
            let matches_symbol = ticker_data.ticker.is_empty()
                || ticker_data.ticker.eq_ignore_ascii_case(symbol);
            if matches_symbol {
                if let Some(bar) = ticker_data.latest_result() {
                    return Some(bar.c);
                }
            }
        }

        if let Some(sym) = crate::app::normalize_symbol(symbol) {
            if let Some(resp) = self.watchlist_quotes.get(&sym) {
                if let Some(bar) = resp.latest_result() {
                    return Some(bar.c);
                }
            }
        }

        if let Some(portfolio_item) = self.portfolio.iter().find(|item| item.symbol == symbol) {
            return portfolio_item.current_price;
        }

        None
    }
}

#[cfg(test)]
mod sanitize_tests {
    use super::sanitize_alert_notify_display_text;

    #[test]
    fn sanitize_plain_symbol() {
        assert_eq!(sanitize_alert_notify_display_text("AAPL"), "AAPL");
    }

    #[test]
    fn sanitize_strips_control_and_collapses_whitespace() {
        assert_eq!(
            sanitize_alert_notify_display_text("AA\nPL"),
            "AA PL"
        );
        assert_eq!(
            sanitize_alert_notify_display_text("MSFT\x00"),
            "MSFT"
        );
    }

    #[test]
    fn sanitize_emptyish() {
        assert_eq!(sanitize_alert_notify_display_text(""), "");
        assert_eq!(sanitize_alert_notify_display_text("  \n\t  "), "");
    }

    #[test]
    fn sanitize_truncates_long_token() {
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let out = sanitize_alert_notify_display_text(s);
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), 32 + 1);
    }
}

#[cfg(test)]
mod notify_body_cap_tests {
    use super::{truncate_utf8_notify_body_to_max_bytes, NOTIFY_BATCH_BODY_MAX_BYTES};

    #[test]
    fn truncate_noop_when_under_cap() {
        assert_eq!(
            truncate_utf8_notify_body_to_max_bytes("a\nb", 1024),
            "a\nb"
        );
    }

    #[test]
    fn truncate_appends_ellipsis_on_byte_cap() {
        let s = "x".repeat(2000);
        let out = truncate_utf8_notify_body_to_max_bytes(&s, 64);
        assert!(out.len() <= 64);
        assert!(out.ends_with('…'));
        assert!(out.starts_with('x'));
    }

    #[test]
    fn truncate_utf8_scalar_boundary() {
        let s = "€".repeat(400);
        let out = truncate_utf8_notify_body_to_max_bytes(&s, 20);
        assert!(out.len() <= 20);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn default_cap_constant_matches_spec() {
        assert_eq!(NOTIFY_BATCH_BODY_MAX_BYTES, 1024);
    }
}
