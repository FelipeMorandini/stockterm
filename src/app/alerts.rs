use crate::app::keyboard::letter_key_plain;
use crate::app::layout::centered_rect;
use crate::app::{AlertAddDialog, AlertAddField, App};
use crate::models::alerts::{process_alert_crossings, Alert, AlertCondition};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use std::io::{self, Write};

const MAX_ALERT_FIELD_LEN: usize = 24;

fn ring_terminal_bell() {
    let mut out = io::stdout();
    let _ = out.write_all(b"\x07");
    let _ = out.flush();
}

#[cfg(feature = "desktop-notify")]
fn spawn_desktop_alert_notification(
    symbol: String,
    condition: AlertCondition,
    threshold: f64,
    last_price: Option<f64>,
) {
    let cond_s = match condition {
        AlertCondition::Above => "Above",
        AlertCondition::Below => "Below",
    };
    std::thread::spawn(move || {
        let mut body = format!("{symbol} {cond_s} ${threshold:.2}");
        if let Some(p) = last_price {
            body.push_str(&format!(" · last ${p:.2}"));
        }
        let show_result = notify_rust::Notification::new()
            .summary("StockTerm")
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

pub fn draw_alerts(f: &mut Frame, app: &mut App, area: Rect) {
    if app.alerts.is_empty() && app.alert_add_dialog.is_none() {
        let block = Block::default()
            .title("Price Alerts")
            .borders(Borders::ALL);
        let no_data_text = Line::from(vec![Span::styled(
            "No alerts configured. Add with a or A (Shift+a works).",
            Style::default().fg(Color::Yellow),
        )]);
        let paragraph = Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    if app.alerts.is_empty() {
        let block = Block::default()
            .title("Price Alerts")
            .borders(Borders::ALL);
        let no_data_text = Line::from(vec![Span::styled(
            "No alerts configured. Add with a or A (Shift+a works).",
            Style::default().fg(Color::Yellow),
        )]);
        let paragraph = Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
    } else {
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let header_cells = ["Symbol", "Condition", "Price", "Current", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
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
                ("TRIGGERED", Color::Red)
            } else if current_opt.is_some() {
                ("Armed", Color::Yellow)
            } else {
                ("No quote", Color::DarkGray)
            };

            let cells = [
                Cell::from(alert.symbol.clone()),
                Cell::from(condition_text),
                Cell::from(format!("${:.2}", alert.price)),
                Cell::from(current_cell),
                Cell::from(status_text).style(Style::default().fg(status_color)),
            ];

            Row::new(cells).height(1)
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
                .title("Price Alerts"),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");

        f.render_stateful_widget(table, area, &mut app.alerts_state);
    }

    if app.alert_add_dialog.is_some() {
        draw_alert_add_overlay(f, app, area);
    }
}

fn draw_alert_add_overlay(f: &mut Frame, app: &App, area: Rect) {
    let Some(dialog) = app.alert_add_dialog.as_ref() else {
        return;
    };

    f.render_widget(Clear, area);

    let popup = centered_rect(area, 55, 42);

    let sym_style = if dialog.focused == AlertAddField::Symbol {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let cond_style = if dialog.focused == AlertAddField::Condition {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let thr_style = if dialog.focused == AlertAddField::Threshold {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let cond_label = match dialog.condition {
        AlertCondition::Above => "Above",
        AlertCondition::Below => "Below",
    };

    let mut lines: Vec<Line> = vec![
        Line::from("Add price alert — Esc cancel · Tab / Shift+Tab or ; cycle field · ←/→ on Condition (Below/Above) · Enter advances / saves on Threshold"),
        Line::from(vec![
            Span::styled("Symbol:    ", sym_style),
            Span::raw(dialog.symbol_buffer.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Condition: ", cond_style),
            Span::raw(cond_label),
            Span::styled("  (; toggles · ← Below · → Above · a/A · b/B)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Threshold: ", thr_style),
            Span::raw("$"),
            Span::raw(dialog.threshold_buffer.as_str()),
        ]),
    ];

    if let Some(ref err) = dialog.inline_error {
        lines.push(Line::from(vec![Span::styled(
            err.as_str(),
            Style::default().fg(Color::Red),
        )]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add alert");
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
            for idx in &newly {
                let alert = &self.alerts[*idx];
                let last = prices
                    .iter()
                    .find(|(s, _)| s == &alert.symbol)
                    .map(|(_, p)| *p);
                spawn_desktop_alert_notification(
                    alert.symbol.clone(),
                    alert.condition,
                    alert.price,
                    last,
                );
            }
        }

        self.save_alerts();
    }

    fn save_alerts(&mut self) {
        self.config.alerts = self.alerts.clone();
        if let Err(e) = self.config.try_save() {
            self.error_message = Some(format!("Failed to save alerts: {e}"));
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
