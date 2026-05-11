use crate::app::keyboard::letter_key_plain;
use crate::app::{normalize_symbol, App, PortfolioAddField, Tab};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

const MAX_HOLDING_INPUT_LEN: usize = 24;

/// Parse a positive decimal for shares or purchase price (Issue #6 / SPEC §13).
pub(crate) fn parse_holding_decimal(input: &str) -> Result<f64, &'static str> {
    let t = input.trim();
    if t.is_empty() {
        return Err("Value required");
    }
    let v: f64 = t.parse().map_err(|_| "Invalid number")?;
    if !v.is_finite() {
        return Err("Invalid number");
    }
    if v <= 0.0 {
        return Err("Must be greater than zero");
    }
    Ok(v)
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y).div_ceil(2)),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x).div_ceil(2)),
        ])
        .split(v[1])[1]
}

fn append_numeric_char(buf: &mut String, c: char) -> bool {
    if buf.len() >= MAX_HOLDING_INPUT_LEN {
        return false;
    }
    if c == '.' {
        if buf.contains('.') {
            return false;
        }
        buf.push('.');
        return true;
    }
    if c.is_ascii_digit() {
        buf.push(c);
        return true;
    }
    false
}

fn portfolio_move_up(app: &mut App) {
    if app.portfolio.is_empty() {
        return;
    }
    match app.portfolio_state.selected() {
        None => app
            .portfolio_state
            .select(Some(app.portfolio.len().saturating_sub(1))),
        Some(i) if i > 0 => app.portfolio_state.select(Some(i - 1)),
        _ => {}
    }
}

fn portfolio_move_down(app: &mut App) {
    if app.portfolio.is_empty() {
        return;
    }
    match app.portfolio_state.selected() {
        None => app.portfolio_state.select(Some(0)),
        Some(i) if i < app.portfolio.len().saturating_sub(1) => {
            app.portfolio_state.select(Some(i + 1));
        }
        _ => {}
    }
}

pub fn draw_portfolio(f: &mut Frame, app: &mut App, area: Rect) {
    if app.portfolio.is_empty() {
        let block = Block::default()
            .title("Portfolio")
            .borders(Borders::ALL);
        let no_data_text = Line::from(vec![Span::styled(
            "Your portfolio is empty. Press 'a' to add the active symbol (set it on Stock View first).",
            Style::default().fg(Color::Yellow),
        )]);
        let paragraph = Paragraph::new(no_data_text)
            .wrap(Wrap { trim: true })
            .block(block);
        f.render_widget(paragraph, area);
    } else {
        let block = Block::default()
            .title("Portfolio")
            .borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let layout = if app.portfolio_remove_armed {
            vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
        } else {
            vec![Constraint::Length(3), Constraint::Min(0)]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(layout)
            .split(inner);

        let total_value = app.calculate_portfolio_value();
        let total_cost = app.calculate_portfolio_cost();
        let total_profit_loss = total_value - total_cost;
        let profit_loss_percent = if total_cost > 0.0 {
            (total_profit_loss / total_cost) * 100.0
        } else {
            0.0
        };

        let pl_color = if total_profit_loss >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        let summary_text = vec![Line::from(vec![
            Span::raw("Total Value: "),
            Span::styled(
                format!("${:.2}", total_value),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  |  Cost Basis: "),
            Span::styled(
                format!("${:.2}", total_cost),
                Style::default().fg(Color::White),
            ),
            Span::raw("  |  P/L: "),
            Span::styled(
                format!("${:.2} ({:.2}%)", total_profit_loss, profit_loss_percent),
                Style::default().fg(pl_color),
            ),
        ])];

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary"));

        f.render_widget(summary, chunks[0]);

        let table_chunk = chunks[1];

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let header_cells = ["Symbol", "Shares", "Avg Price", "Current", "Value", "P/L", "P/L %"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        let rows = app.portfolio.iter().map(|item| {
            let current_price = item.current_price.unwrap_or(0.0);
            let market_value = current_price * item.shares;
            let profit_loss = market_value - (item.purchase_price * item.shares);
            let pl_percent = if item.purchase_price > 0.0 {
                (profit_loss / (item.purchase_price * item.shares)) * 100.0
            } else {
                0.0
            };

            let pl_color = if profit_loss >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };

            let cells = [
                Cell::from(item.symbol.clone()),
                Cell::from(format!("{:.2}", item.shares)),
                Cell::from(format!("${:.2}", item.purchase_price)),
                Cell::from(format!("${:.2}", current_price)),
                Cell::from(format!("${:.2}", market_value)),
                Cell::from(format!("${:.2}", profit_loss)).style(Style::default().fg(pl_color)),
                Cell::from(format!("{:.2}%", pl_percent)).style(Style::default().fg(pl_color)),
            ];

            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Holdings"))
        .highlight_style(selected_style)
        .highlight_symbol("> ");

        f.render_stateful_widget(table, table_chunk, &mut app.portfolio_state);

        if app.portfolio_remove_armed {
            let hint = Paragraph::new(Line::from(vec![Span::styled(
                "Remove armed — confirm: d or y  |  cancel: Esc or n",
                Style::default().fg(Color::Yellow),
            )]));
            f.render_widget(hint, chunks[2]);
        }
    }

    if app.portfolio_dialog.is_some() {
        draw_portfolio_add_overlay(f, app, area);
    }
}

fn draw_portfolio_add_overlay(f: &mut Frame, app: &App, area: Rect) {
    let Some(dialog) = app.portfolio_dialog.as_ref() else {
        return;
    };

    f.render_widget(Clear, area);

    let popup = centered_rect(area, 55, 40);
    let sym_label = normalize_symbol(&app.symbol).unwrap_or_default();

    let shares_style = if dialog.focused == PortfolioAddField::Shares {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let price_style = if dialog.focused == PortfolioAddField::Price {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mut lines: Vec<Line> = vec![
        Line::from("Add holding (Esc cancel, ; cycles Shares/Price — Tab switches app tab)"),
        Line::from(vec![
            Span::raw("Symbol: "),
            Span::styled(sym_label, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Shares:  ", shares_style),
            Span::raw(dialog.shares_buffer.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Price:   ", price_style),
            Span::raw(dialog.price_buffer.as_str()),
        ]),
        Line::from("Enter on Shares → Price | Enter on Price → save"),
    ];

    if let Some(ref err) = dialog.inline_error {
        lines.push(Line::from(vec![Span::styled(
            err.as_str(),
            Style::default().fg(Color::Red),
        )]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add to portfolio");
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, popup);
}

fn try_commit_portfolio_dialog(app: &mut App) {
    let Some(ref dlg) = app.portfolio_dialog else {
        return;
    };
    let shares_r = parse_holding_decimal(&dlg.shares_buffer);
    let price_r = parse_holding_decimal(&dlg.price_buffer);

    match (shares_r, price_r) {
        (Ok(shares), Ok(price)) => {
            if app.add_to_portfolio(shares, price) {
                app.portfolio_dialog = None;
                app.request_immediate_stock_poll();
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            if let Some(d) = app.portfolio_dialog.as_mut() {
                d.inline_error = Some(e.to_string());
            }
        }
    }
}

fn handle_portfolio_dialog_keys(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    match key.code {
        Esc => {
            app.portfolio_dialog = None;
        }
        // `Tab` is handled globally (app tab switch). Use `;` to cycle fields (SPEC §13.2).
        Char(';') if key.modifiers == KeyModifiers::NONE => {
            if let Some(d) = app.portfolio_dialog.as_mut() {
                d.inline_error = None;
                d.focused = match d.focused {
                    PortfolioAddField::Shares => PortfolioAddField::Price,
                    PortfolioAddField::Price => PortfolioAddField::Shares,
                };
            }
        }
        Backspace if key.modifiers == KeyModifiers::NONE => {
            if let Some(d) = app.portfolio_dialog.as_mut() {
                d.inline_error = None;
                let buf = match d.focused {
                    PortfolioAddField::Shares => &mut d.shares_buffer,
                    PortfolioAddField::Price => &mut d.price_buffer,
                };
                buf.pop();
            }
        }
        Enter if key.modifiers == KeyModifiers::NONE => {
            if let Some(d) = app.portfolio_dialog.as_mut() {
                d.inline_error = None;
                match d.focused {
                    PortfolioAddField::Shares => d.focused = PortfolioAddField::Price,
                    PortfolioAddField::Price => try_commit_portfolio_dialog(app),
                }
            }
        }
        Char(c)
            if key.modifiers == KeyModifiers::NONE && (c.is_ascii_digit() || c == '.') =>
        {
            if let Some(d) = app.portfolio_dialog.as_mut() {
                d.inline_error = None;
                let buf = match d.focused {
                    PortfolioAddField::Shares => &mut d.shares_buffer,
                    PortfolioAddField::Price => &mut d.price_buffer,
                };
                let _ = append_numeric_char(buf, c);
            }
        }
        _ => {}
    }
}

fn handle_portfolio_remove_armed_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.portfolio_remove_armed = false;
        }
        KeyCode::Char(c)
            if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'n') =>
        {
            app.portfolio_remove_armed = false;
        }
        KeyCode::Char(c)
            if letter_key_plain(key.modifiers)
                && (c.eq_ignore_ascii_case(&'d') || c.eq_ignore_ascii_case(&'y')) =>
        {
            if let Some(selected) = app.portfolio_state.selected() {
                if app.remove_from_portfolio(selected) {
                    app.portfolio_remove_armed = false;
                }
            } else {
                app.portfolio_remove_armed = false;
            }
        }
        KeyCode::Up => portfolio_move_up(app),
        KeyCode::Down => portfolio_move_down(app),
        KeyCode::Char(c) if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'j') => {
            portfolio_move_down(app);
        }
        KeyCode::Char(c) if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'k') => {
            portfolio_move_up(app);
        }
        _ => {}
    }
}

pub fn handle_portfolio_events(app: &mut App, key: KeyEvent) {
    if app.portfolio_dialog.is_some() {
        handle_portfolio_dialog_keys(app, key);
        return;
    }

    if app.portfolio_remove_armed {
        handle_portfolio_remove_armed_keys(app, key);
        return;
    }

    match key {
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'a') => {
            if normalize_symbol(&app.symbol).is_none() {
                app.error_message = Some(
                    "Set a ticker on Stock View first (or press Enter on a holding).".to_string(),
                );
                return;
            }
            app.portfolio_remove_armed = false;
            app.portfolio_dialog = Some(crate::app::PortfolioAddDialog::default());
            app.error_message = None;
        }
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'d') => {
            if app.portfolio.is_empty() {
                return;
            }
            if app.portfolio_state.selected().is_none() {
                app.portfolio_state.select(Some(0));
            }
            app.portfolio_remove_armed = true;
        }
        KeyEvent {
            code: KeyCode::Up,
            ..
        } => portfolio_move_up(app),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => portfolio_move_down(app),
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'j') => {
            portfolio_move_down(app);
        }
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } if letter_key_plain(key.modifiers) && c.eq_ignore_ascii_case(&'k') => {
            portfolio_move_up(app);
        }
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            if let Some(selected) = app.portfolio_state.selected() {
                if selected < app.portfolio.len() {
                    app.symbol = app.portfolio[selected].symbol.clone();
                    app.on_active_symbol_changed_for_charts();
                    app.notify_symbol_changed_for_news();
                    app.sync_watchlist_selection_to_symbol();
                    app.request_immediate_stock_poll();
                    app.active_tab = Tab::StockView;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::parse_holding_decimal;

    #[test]
    fn parse_holding_decimal_accepts_positive() {
        assert!((parse_holding_decimal("10").unwrap() - 10.0).abs() < f64::EPSILON);
        assert!((parse_holding_decimal("412.55").unwrap() - 412.55).abs() < 1e-9);
        assert!((parse_holding_decimal("  1.5  ").unwrap() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_holding_decimal_rejects_invalid() {
        assert!(parse_holding_decimal("").is_err());
        assert!(parse_holding_decimal("   ").is_err());
        assert!(parse_holding_decimal("0").is_err());
        assert!(parse_holding_decimal("-1").is_err());
        assert!(parse_holding_decimal("abc").is_err());
    }
}
