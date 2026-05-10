use crate::app::alerts::draw_alerts;
use crate::app::charts::draw_charts;
use crate::app::portfolio::draw_portfolio;
use crate::app::{App, Tab};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
    Terminal,
};
use std::io;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(size);

        let titles = [
            "Stock View",
            "Portfolio",
            "Alerts",
            "Search",
            "News",
            "Charts",
            "Settings",
        ];

        let tabs = Tabs::new(titles.iter().map(|t| Line::from(*t)).collect())
            .block(Block::default().borders(Borders::ALL).title("StockTerm"))
            .select(match app.active_tab {
                Tab::StockView => 0,
                Tab::Portfolio => 1,
                Tab::Alerts => 2,
                Tab::Search => 3,
                Tab::News => 4,
                Tab::Charts => 5,
                Tab::Settings => 6,
            })
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(tabs, chunks[0]);

        match app.active_tab {
            Tab::StockView => draw_stock_view(f, app, chunks[1]),
            Tab::Portfolio => draw_portfolio(f, app, chunks[1]),
            Tab::Alerts => draw_alerts(f, app, chunks[1]),
            Tab::Search => draw_search(f, app, chunks[1]),
            Tab::News => draw_news(f, app, chunks[1]),
            Tab::Charts => draw_charts(f, app, chunks[1]),
            Tab::Settings => draw_settings(f, app, chunks[1]),
        }

        draw_status_bar(f, app, chunks[2]);
    })?;
    Ok(())
}

fn draw_stock_view(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Stock Info: {}", app.symbol))
        .borders(Borders::ALL);

    if let Some(ticker_data) = &app.ticker_data {
        if ticker_data.results.is_empty() {
            let text = vec![Line::from(vec![Span::styled(
                "No quote data returned for this symbol.",
                Style::default().fg(Color::Yellow),
            )])];
            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, area);
            return;
        }

        {
            let Some(result) = ticker_data.latest_result() else {
                let text = vec![Line::from(vec![Span::styled(
                    "No quote data returned for this symbol.",
                    Style::default().fg(Color::Yellow),
                )])];
                let paragraph = Paragraph::new(text).block(block);
                f.render_widget(paragraph, area);
                return;
            };
            let price_change = result.c - result.o;
            let percent_change = (price_change / result.o) * 100.0;
            let change_color = if price_change >= 0.0 {
                Color::Green
            } else {
                Color::Red
            };

            let text = vec![
                Line::from(vec![
                    Span::raw("Symbol: "),
                    Span::styled(&app.symbol, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Price: "),
                    Span::styled(
                        format!("${:.2}", result.c),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Change: "),
                    Span::styled(
                        format!(
                            "{}{:.2} ({:.2}%)",
                            if price_change >= 0.0 { "+" } else { "" },
                            price_change,
                            percent_change
                        ),
                        Style::default().fg(change_color),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Open: "),
                    Span::styled(
                        format!("${:.2}", result.o),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("High: "),
                    Span::styled(
                        format!("${:.2}", result.h),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Low: "),
                    Span::styled(
                        format!("${:.2}", result.l),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Volume: "),
                    Span::styled(
                        format!("{:.0}", result.v),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ];

            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, area);
        }
    } else if let Some(error) = &app.error_message {
        let text = vec![Line::from(vec![Span::styled(
            error,
            Style::default().fg(Color::Red),
        )])];
        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    } else {
        let text = vec![Line::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])];
        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }
}

fn draw_search(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_news(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_settings(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(error) = &app.error_message {
        Line::from(vec![Span::styled(error, Style::default().fg(Color::Red))])
    } else {
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" to quit, "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" to switch tabs"),
        ])
    };

    let paragraph = Paragraph::new(status);
    f.render_widget(paragraph, area);
}
