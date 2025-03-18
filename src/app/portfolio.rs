use crate::app::App;
use crate::models::portfolio::PortfolioItem;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Frame,
};
use std::format;
use std::vec;

pub fn draw_portfolio(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Portfolio")
        .borders(Borders::ALL);

    if app.portfolio.is_empty() {
        let no_data_text = Line::from(vec![
            Span::styled("Your portfolio is empty. Add stocks with the 'a' key.", Style::default().fg(Color::Yellow))
        ]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text)
            .block(block);
        f.render_widget(paragraph, area);
        return;
    }

    // Create portfolio summary and details layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Summary
            Constraint::Min(0),     // Details
        ].as_ref())
        .split(area);

    // Render portfolio summary
    let total_value = app.calculate_portfolio_value();
    let total_cost = app.calculate_portfolio_cost();
    let total_profit_loss = total_value - total_cost;
    let profit_loss_percent = if total_cost > 0.0 {
        (total_profit_loss / total_cost) * 100.0
    } else {
        0.0
    };

    let pl_color = if total_profit_loss >= 0.0 { Color::Green } else { Color::Red };

    let summary_text = vec![
        Line::from(vec![
            Span::raw("Total Value: "),
            Span::styled(format!("${:.2}", total_value), Style::default().fg(Color::Cyan)),
            Span::raw("  |  Cost Basis: "),
            Span::styled(format!("${:.2}", total_cost), Style::default().fg(Color::White)),
            Span::raw("  |  P/L: "),
            Span::styled(
                format!("${:.2} ({:.2}%)", total_profit_loss, profit_loss_percent),
                Style::default().fg(pl_color)
            ),
        ]),
    ];

    let summary = ratatui::widgets::Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Summary"))
        .style(Style::default());

    f.render_widget(summary, chunks[0]);

    // Render portfolio details table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = ["Symbol", "Shares", "Avg Price", "Current", "Value", "P/L", "P/L %"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = app.portfolio.iter().enumerate().map(|(i, item)| {
        let current_price = item.current_price.unwrap_or(0.0);
        let market_value = current_price * item.shares;
        let profit_loss = market_value - (item.purchase_price * item.shares);
        let pl_percent = if item.purchase_price > 0.0 {
            (profit_loss / (item.purchase_price * item.shares)) * 100.0
        } else {
            0.0
        };

        let pl_color = if profit_loss >= 0.0 { Color::Green } else { Color::Red };

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
        .block(Block::default().borders(Borders::ALL).title("Holdings"))
        .highlight_style(selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(table, chunks[1], &mut app.portfolio_state);
}

pub fn handle_portfolio_events(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key {
        crossterm::event::KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Add current symbol to portfolio
            // In a real app, you'd show a dialog to enter shares and price
            app.add_to_portfolio(1.0, 100.0);
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Remove selected item from portfolio
            if let Some(selected) = app.portfolio_state.selected() {
                app.remove_from_portfolio(selected);
            }
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Move selection up
            let current = app.portfolio_state.selected().unwrap_or(0);
            if current > 0 {
                app.portfolio_state.select(Some(current - 1));
            }
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Move selection down
            let current = app.portfolio_state.selected().unwrap_or(0);
            if current < app.portfolio.len().saturating_sub(1) {
                app.portfolio_state.select(Some(current + 1));
            }
        }
        crossterm::event::KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Select the highlighted portfolio item
            if let Some(selected) = app.portfolio_state.selected() {
                if selected < app.portfolio.len() {
                    app.symbol = app.portfolio[selected].symbol.clone();
                    app.fetch_ticker_data();
                    app.active_tab = crate::app::app::Tab::StockView;
                }
            }
        }
        _ => {}
    }
}
