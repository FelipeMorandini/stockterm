use crate::app::App;
use crate::models::alerts::{Alert, AlertCondition};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::format;
use std::vec;


pub fn draw_alerts(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Price Alerts")
        .borders(Borders::ALL);

    if app.alerts.is_empty() {
        let no_data_text = Line::from(vec![
            Span::styled("No alerts configured. Add alerts with the 'a' key.", Style::default().fg(Color::Yellow))
        ]);
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

    let rows = app.alerts.iter().enumerate().map(|(i, alert)| {
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
            Constraint::Length(8),    // Symbol
            Constraint::Length(8),    // Shares
            Constraint::Length(10),   // Avg Price
            Constraint::Length(10),   // Current
            Constraint::Length(10),   // Value
            Constraint::Length(10),   // P/L
            Constraint::Length(10),   // P/L %
        ]
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Alerts"))
        .highlight_style(selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.alerts_state);
}pub fn handle_alerts_events(app: &mut App, key: crossterm::event::KeyEvent) {    use crossterm::event::{KeyCode, KeyModifiers};

    match key {
        crossterm::event::KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Add new alert for current symbol
            // In a real app, you'd show a dialog to enter price and condition
            app.add_alert(app.symbol.clone(), AlertCondition::Above, 100.0);
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Remove selected alert
            if let Some(selected) = app.alerts_state.selected() {
                app.remove_alert(selected);
            }
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Move selection up
            let current = app.alerts_state.selected().unwrap_or(0);
            if current > 0 {
                app.alerts_state.select(Some(current - 1));
            }
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Move selection down
            let current = app.alerts_state.selected().unwrap_or(0);
            if current < app.alerts.len().saturating_sub(1) {
                app.alerts_state.select(Some(current + 1));
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

        // Save to config
        self.save_alerts();
    }

    pub fn remove_alert(&mut self, index: usize) {
        if index < self.alerts.len() {
            self.alerts.remove(index);

            // Save to config
            self.save_alerts();
        }
    }

    pub fn check_alerts(&mut self) {
        for alert in &mut self.alerts {
            if let Some(current_price) = self.get_current_price(&alert.symbol) {
                match alert.condition {
                    AlertCondition::Above => {
                        if current_price > alert.price {
                            alert.triggered = true;
                        }
                    },
                    AlertCondition::Below => {
                        if current_price < alert.price {
                            alert.triggered = true;
                        }
                    },
                }
            }
        }

        // Save triggered state
        self.save_alerts();
    }

    fn save_alerts(&self) {
        // In a real implementation, you'd save to config
        // self.config.alerts = self.alerts.clone();
        // self.config.save();
    }

    pub fn get_current_price(&self, symbol: &str) -> Option<f64> {
        // If the symbol matches the current ticker data, use that
        if let Some(ticker_data) = &self.ticker_data {
            if ticker_data.ticker == symbol && !ticker_data.results.is_empty() {
                return Some(ticker_data.results[0].c);
            }
        }

        // Otherwise check if it's in the portfolio with a current price
        if let Some(portfolio_item) = self.portfolio.iter().find(|item| item.symbol == symbol) {
            return portfolio_item.current_price;
        }

        None
    }
}
