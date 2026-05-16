#![allow(clippy::collapsible_match, clippy::needless_return)]

use crate::app::styles::ResolvedTheme;
use crate::app::app_error::{AppError, ErrorSourceDomain};
use crate::app::keyboard::letter_key_plain;
use crate::app::layout::centered_rect;
use crate::app::table_filter::filter_title_suffix;
use crate::app::{normalize_symbol, App, PortfolioAddField, Tab};
use crate::config::keymap::{Action, BindingLayer};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

const MAX_HOLDING_INPUT_LEN: usize = 24;

/// Upper sanity bound for shares (paste / typo); SPEC §15.5.
pub(crate) const MAX_HOLDING_SHARES: f64 = 1_000_000_000.0;
/// Upper sanity bound for price per share; SPEC §15.5.
pub(crate) const MAX_HOLDING_PRICE_PER_SHARE: f64 = 1e12;

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

pub(crate) fn validate_holding_limits(shares: f64, price: f64) -> Result<(), &'static str> {
    if shares > MAX_HOLDING_SHARES {
        return Err("Shares exceed the allowed maximum");
    }
    if price > MAX_HOLDING_PRICE_PER_SHARE {
        return Err("Price per share exceeds the allowed maximum");
    }
    Ok(())
}

/// Cycle Shares ↔ Price when the add dialog is open (#67 / SPEC §15.4). With only two
/// fields, forward and backward are the same toggle.
pub(crate) fn cycle_portfolio_dialog_focus(app: &mut App, _forward: bool) {
    let Some(d) = app.portfolio_dialog.as_mut() else {
        return;
    };
    d.inline_error = None;
    d.focused = match d.focused {
        PortfolioAddField::Shares => PortfolioAddField::Price,
        PortfolioAddField::Price => PortfolioAddField::Shares,
    };
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
    let f = app.portfolio_filter_indices();
    if f.is_empty() {
        return;
    }
    match app.portfolio_state.selected() {
        None => app
            .portfolio_state
            .select(Some(f.len().saturating_sub(1))),
        Some(i) if i > 0 => app.portfolio_state.select(Some(i - 1)),
        _ => {}
    }
}

fn portfolio_move_down(app: &mut App) {
    let f = app.portfolio_filter_indices();
    if f.is_empty() {
        return;
    }
    match app.portfolio_state.selected() {
        None => app.portfolio_state.select(Some(0)),
        Some(i) if i < f.len().saturating_sub(1) => {
            app.portfolio_state.select(Some(i + 1));
        }
        _ => {}
    }
}

pub fn draw_portfolio(f: &mut Frame, app: &mut App, area: Rect, theme: ResolvedTheme) {
    let border_st = Style::default().fg(theme.border).bg(theme.background);
    if app.portfolio.is_empty() {
        let block = Block::default()
            .title("Portfolio")
            .borders(Borders::ALL)
            .style(theme.canvas())
            .border_style(border_st);
        let no_data_text = Line::from(vec![Span::styled(
            "Your portfolio is empty. Press 'a' to add the active symbol (set it on Stock View first).",
            theme.fg_border(),
        )]);
        let paragraph = Paragraph::new(no_data_text)
            .wrap(Wrap { trim: true })
            .block(block);
        f.render_widget(paragraph, area);
    } else {
        let block = Block::default()
            .title("Portfolio")
            .borders(Borders::ALL)
            .style(theme.canvas())
            .border_style(border_st);

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
            theme.positive
        } else {
            theme.negative
        };

        let summary_text = vec![Line::from(vec![
            Span::styled("Total Value: ", theme.canvas()),
            Span::styled(
                format!("${:.2}", total_value),
                theme.fg_accent(),
            ),
            Span::styled("  |  Cost Basis: ", theme.canvas()),
            Span::styled(
                format!("${:.2}", total_cost),
                theme.fg_foreground(),
            ),
            Span::styled("  |  P/L: ", theme.canvas()),
            Span::styled(
                format!("${:.2} ({:.2}%)", total_profit_loss, profit_loss_percent),
                theme.fg_color(pl_color),
            ),
        ])];

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Summary")
                    .style(theme.canvas())
                    .border_style(border_st),
            );

        f.render_widget(summary, chunks[0]);

        let table_chunk = chunks[1];

        let selected_style = Style::default()
            .bg(theme.selection)
            .fg(theme.foreground)
            .add_modifier(Modifier::BOLD);

        let header_cells = ["Symbol", "Shares", "Avg Price", "Current", "Value", "P/L", "P/L %"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.fg_foreground()));

        let header = Row::new(header_cells)
            .style(theme.canvas().add_modifier(Modifier::BOLD))
            .height(1);

        let holdings_title = format!("Holdings{}", filter_title_suffix(&app.filter_query));

        let filtered_idx = app.portfolio_filter_indices();
        if filtered_idx.is_empty() {
            let empty = Line::from(vec![Span::styled(
                "No symbols match filter — press Esc to clear.",
                theme.fg_border(),
            )]);
            let table = Paragraph::new(empty).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(holdings_title)
                    .style(theme.canvas())
                    .border_style(border_st),
            );
            f.render_widget(table, table_chunk);
        } else {
            let rows = filtered_idx.iter().map(|&idx| {
                let item = &app.portfolio[idx];
                let current_price = item.current_price.unwrap_or(0.0);
                let market_value = current_price * item.shares;
                let profit_loss = market_value - (item.purchase_price * item.shares);
                let pl_percent = if item.purchase_price > 0.0 {
                    (profit_loss / (item.purchase_price * item.shares)) * 100.0
                } else {
                    0.0
                };

                let pl_color = if profit_loss >= 0.0 {
                    theme.positive
                } else {
                    theme.negative
                };

                let cells = [
                    Cell::from(item.symbol.clone()),
                    Cell::from(format!("{:.2}", item.shares)),
                    Cell::from(format!("${:.2}", item.purchase_price)),
                    Cell::from(format!("${:.2}", current_price)),
                    Cell::from(format!("${:.2}", market_value)),
                    Cell::from(format!("${:.2}", profit_loss)).style(theme.fg_color(pl_color)),
                    Cell::from(format!("{:.2}%", pl_percent)).style(theme.fg_color(pl_color)),
                ];

                Row::new(cells).height(1).style(theme.canvas())
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(holdings_title)
                    .style(theme.canvas())
                    .border_style(border_st),
            )
            .highlight_style(selected_style)
            .highlight_symbol("> ");

            f.render_stateful_widget(table, table_chunk, &mut app.portfolio_state);
        }

        if app.portfolio_remove_armed {
            let hint = Paragraph::new(Line::from(vec![Span::styled(
                "Remove armed — confirm: d or y  |  cancel: Esc or n",
                theme.fg_border(),
            )]))
            .style(theme.canvas());
            f.render_widget(hint, chunks[2]);
        }
    }

    if app.portfolio_dialog.is_some() {
        draw_portfolio_add_overlay(f, app, area, theme);
    }
}

fn draw_portfolio_add_overlay(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let Some(dialog) = app.portfolio_dialog.as_ref() else {
        return;
    };

    f.render_widget(Clear, area);

    let popup = centered_rect(area, 55, 40);
    let border_st = Style::default().fg(theme.border).bg(theme.background);
    let sym_label = normalize_symbol(&app.symbol).unwrap_or_default();

    let shares_style = if dialog.focused == PortfolioAddField::Shares {
        theme.fg_accent()
    } else {
        theme.fg_foreground()
    };
    let price_style = if dialog.focused == PortfolioAddField::Price {
        theme.fg_accent()
    } else {
        theme.fg_foreground()
    };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(
            "Add holding — Esc cancel · Tab / Shift+Tab or ; cycle field · Enter on Price saves",
            theme.canvas(),
        )]),
        Line::from(vec![
            Span::styled("Symbol: ", theme.canvas()),
            Span::styled(sym_label, theme.fg_accent().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Shares:  ", shares_style),
            Span::styled(dialog.shares_buffer.as_str(), theme.fg_foreground()),
        ]),
        Line::from(vec![
            Span::styled("Price:   ", price_style),
            Span::styled(dialog.price_buffer.as_str(), theme.fg_foreground()),
        ]),
        Line::from(vec![Span::styled(
            "Enter on Shares → Price | Enter on Price → save",
            theme.fg_muted(),
        )]),
    ];

    if let Some(ref err) = dialog.inline_error {
        lines.push(Line::from(vec![Span::styled(
            err.as_str(),
            theme.error_text(),
        )]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add to portfolio")
        .style(theme.canvas())
        .border_style(border_st);
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, popup);
}

pub(crate) fn try_commit_portfolio_dialog(app: &mut App) {
    let Some(ref dlg) = app.portfolio_dialog else {
        return;
    };
    let shares_r = parse_holding_decimal(&dlg.shares_buffer);
    let price_r = parse_holding_decimal(&dlg.price_buffer);

    match (shares_r, price_r) {
        (Ok(shares), Ok(price)) => {
            if let Err(e) = validate_holding_limits(shares, price) {
                if let Some(d) = app.portfolio_dialog.as_mut() {
                    d.inline_error = Some(e.to_string());
                }
                return;
            }
            if app.add_to_portfolio(shares, price) {
                app.portfolio_dialog = None;
                app.request_immediate_stock_poll();
            } else if app.error_message().is_none() {
                if let Some(d) = app.portfolio_dialog.as_mut() {
                    d.inline_error = Some(
                        "Cannot add holding: no valid ticker is set. Pick a symbol on Stock View."
                            .into(),
                    );
                }
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
    if let Some(a) = app
        .resolved_keymap
        .action(BindingLayer::PortfolioDialog, &key)
    {
        match a {
            Action::PortfolioDialogEsc if key.modifiers == KeyModifiers::NONE => {
                app.portfolio_dialog = None;
                return;
            }
            Action::PortfolioDialogFocusNext if key.modifiers == KeyModifiers::NONE => {
                cycle_portfolio_dialog_focus(app, true);
                return;
            }
            Action::PortfolioDialogBackspace if key.modifiers == KeyModifiers::NONE => {
                if let Some(d) = app.portfolio_dialog.as_mut() {
                    d.inline_error = None;
                    let buf = match d.focused {
                        PortfolioAddField::Shares => &mut d.shares_buffer,
                        PortfolioAddField::Price => &mut d.price_buffer,
                    };
                    buf.pop();
                }
                return;
            }
            Action::PortfolioDialogEnter if key.modifiers == KeyModifiers::NONE => {
                if let Some(d) = app.portfolio_dialog.as_mut() {
                    d.inline_error = None;
                    match d.focused {
                        PortfolioAddField::Shares => d.focused = PortfolioAddField::Price,
                        PortfolioAddField::Price => try_commit_portfolio_dialog(app),
                    }
                }
                return;
            }
            Action::PortfolioDialogDigitOrDot => {
                if !letter_key_plain(key.modifiers) {
                    return;
                }
                let KeyCode::Char(c) = key.code else {
                    return;
                };
                if let Some(d) = app.portfolio_dialog.as_mut() {
                    d.inline_error = None;
                    let buf = match d.focused {
                        PortfolioAddField::Shares => &mut d.shares_buffer,
                        PortfolioAddField::Price => &mut d.price_buffer,
                    };
                    let _ = append_numeric_char(buf, c);
                }
                return;
            }
            _ => {}
        }
    }
}

fn handle_portfolio_remove_armed_keys(app: &mut App, key: KeyEvent) {
    if let Some(a) = app
        .resolved_keymap
        .action(BindingLayer::PortfolioRemoveArmed, &key)
    {
        match a {
            Action::PortfolioRemoveCancel => {
                app.portfolio_remove_armed = false;
                return;
            }
            Action::PortfolioRemoveDecline => {
                app.portfolio_remove_armed = false;
                return;
            }
            Action::PortfolioRemoveConfirm => {
                if let Some(selected_f) = app.portfolio_state.selected() {
                    let filtered = app.portfolio_filter_indices();
                    if selected_f < filtered.len() {
                        let actual = filtered[selected_f];
                        if app.remove_from_portfolio(actual) {
                            app.portfolio_remove_armed = false;
                        }
                    } else {
                        app.portfolio_remove_armed = false;
                    }
                } else {
                    app.portfolio_remove_armed = false;
                }
                return;
            }
            Action::PortfolioRowUp => {
                portfolio_move_up(app);
                return;
            }
            Action::PortfolioRowDown => {
                portfolio_move_down(app);
                return;
            }
            _ => {}
        }
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

    if app.consume_filter_input_key(&key) {
        return;
    }

    if let Some(a) = app.resolved_keymap.action(BindingLayer::Portfolio, &key) {
        match a {
            Action::PortfolioFilterToggle if key.modifiers == KeyModifiers::NONE => {
                app.filter_input_mode = true;
            }
            Action::PortfolioAdd if letter_key_plain(key.modifiers) => {
                if normalize_symbol(&app.symbol).is_none() {
                    app.surface_runtime_error(
                        Tab::Portfolio,
                        ErrorSourceDomain::Portfolio,
                        AppError::Internal(
                            "Set a ticker on Stock View first (or press Enter on a holding)."
                                .to_string(),
                        ),
                        true,
                    );
                    return;
                }
                app.portfolio_remove_armed = false;
                app.portfolio_dialog = Some(crate::app::PortfolioAddDialog::default());
                app.clear_active_runtime_unless_alerts_save();
            }
            Action::PortfolioRemoveArm if letter_key_plain(key.modifiers) => {
                if app.portfolio.is_empty() {
                    return;
                }
                if app.portfolio_state.selected().is_none() {
                    app.portfolio_state.select(Some(0));
                }
                app.portfolio_remove_armed = true;
            }
            Action::PortfolioRowUp => {
                portfolio_move_up(app);
            }
            Action::PortfolioRowDown => {
                portfolio_move_down(app);
            }
            Action::PortfolioEnterStock if key.modifiers == KeyModifiers::NONE => {
                if let Some(selected_f) = app.portfolio_state.selected() {
                    let filtered = app.portfolio_filter_indices();
                    if selected_f < filtered.len() {
                        let idx = filtered[selected_f];
                        app.symbol = app.portfolio[idx].symbol.clone();
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
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_holding_decimal, validate_holding_limits, MAX_HOLDING_PRICE_PER_SHARE,
        MAX_HOLDING_SHARES,
    };

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

    #[test]
    fn validate_holding_limits_accepts_at_ceiling() {
        assert!(validate_holding_limits(MAX_HOLDING_SHARES, MAX_HOLDING_PRICE_PER_SHARE).is_ok());
    }

    #[test]
    fn validate_holding_limits_rejects_above_ceiling() {
        assert!(validate_holding_limits(MAX_HOLDING_SHARES * 2.0, 1.0).is_err());
        assert!(validate_holding_limits(1.0, MAX_HOLDING_PRICE_PER_SHARE * 2.0).is_err());
    }
}
