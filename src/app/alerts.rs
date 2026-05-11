use crate::app::App;
use crate::models::alerts::{Alert, AlertCondition};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn draw_alerts(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Price Alerts")
        .borders(Borders::ALL);

    if app.alerts.is_empty() {
        let no_data_text = Line::from(vec![Span::styled(
            "No alerts configured. Add with a or A (Shift+a works).",
            Style::default().fg(Color::Yellow),
        )]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text)
            .block(block);
        f.render_widget(paragraph, area);
        return;
    }

    // Render alerts table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = ["Symbol", "Condition", "Price", "Current", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = app.alerts.iter().map(|alert| {
        // Get current price for the alert symbol
        let current_price = app.get_current_price(&alert.symbol).unwrap_or(0.0);

        // Determine if alert is triggered
        let is_triggered = match alert.condition {
            AlertCondition::Above => current_price > alert.price,
            AlertCondition::Below => current_price < alert.price,
        };

        let condition_text = match alert.condition {
            AlertCondition::Above => "Above",
            AlertCondition::Below => "Below",
        };

        let status_text = if is_triggered { "TRIGGERED" } else { "Waiting" };
        let status_color = if is_triggered { Color::Red } else { Color::Yellow };

        let cells = [
            Cell::from(alert.symbol.clone()),
            Cell::from(condition_text),
            Cell::from(format!("${:.2}", alert.price)),
            Cell::from(format!("${:.2}", current_price)),
            Cell::from(status_text).style(Style::default().fg(status_color)),
        ];

        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(6),       // Symbol
            Constraint::Length(10),   // Condition (Above / Below)
            Constraint::Length(11),   // Price
            Constraint::Length(11),   // Current
            Constraint::Min(10),      // Status (Waiting / TRIGGERED)
        ],
    )
        .header(header)
        // Same pane title as empty state (#43 / SPEC §15.1).
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Price Alerts"),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.alerts_state);
}

pub fn handle_alerts_events(app: &mut App, key: crossterm::event::KeyEvent) {
    use crate::app::keyboard::letter_key_plain;
    use crossterm::event::KeyCode;

    match key {
        crossterm::event::KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.eq_ignore_ascii_case(&'a') && letter_key_plain(modifiers) => {
            app.add_alert(app.symbol.clone(), AlertCondition::Above, 100.0);
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            ..
        } if c.eq_ignore_ascii_case(&'d') && letter_key_plain(modifiers) => {
            if let Some(selected) = app.alerts_state.selected() {
                app.remove_alert(selected);
            }
        }
        crossterm::event::KeyEvent {
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
        crossterm::event::KeyEvent {
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

// Add these methods to your App struct
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
        // First, collect all the symbols and their current prices
        let prices: Vec<(String, f64)> = self.alerts
            .iter()
            .filter_map(|alert| {
                self.get_current_price(&alert.symbol)
                    .map(|price| (alert.symbol.clone(), price))
            })
            .collect();

        // Then update the alerts with the collected prices
        let mut updated = false;
        for alert in &mut self.alerts {
            if let Some((_, price)) = prices.iter().find(|(symbol, _)| symbol == &alert.symbol) {
                let is_triggered = match alert.condition {
                    AlertCondition::Above => *price > alert.price,
                    AlertCondition::Below => *price < alert.price,
                };

                if is_triggered && !alert.triggered {
                    alert.triggered = true;
                    updated = true;
                }
            }
        }

        if updated {
            self.save_alerts();
        }
    }

    fn save_alerts(&mut self) {
        self.config.alerts = self.alerts.clone();
        if let Err(e) = self.config.try_save() {
            self.error_message = Some(format!("Failed to save alerts: {e}"));
        }
    }

    pub fn get_current_price(&self, symbol: &str) -> Option<f64> {
        // If the symbol matches the current ticker data, use that
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

        // Otherwise check if it's in the portfolio with a current price
        if let Some(portfolio_item) = self.portfolio.iter().find(|item| item.symbol == symbol) {
            return portfolio_item.current_price;
        }

        None
    }
}
