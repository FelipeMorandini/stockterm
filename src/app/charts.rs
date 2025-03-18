use crate::app::App;
use crate::models::historical::HistoricalData;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Line},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use std::format;
use std::vec;


pub fn draw_charts(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Price Chart: {}", app.symbol))
        .borders(Borders::ALL);

    if let Some(historical_data) = &app.historical_data {
        if historical_data.results.is_empty() {
            let no_data_text = Line::from(vec![
                Span::styled("No historical data available", Style::default().fg(Color::Red))
            ]);
            let paragraph = ratatui::widgets::Paragraph::new(no_data_text)
                .block(block);
            f.render_widget(paragraph, area);
            return;
        }

        // Prepare data for chart
        let data: Vec<(f64, f64)> = historical_data.results.iter()
            .map(|data| {
                // Convert timestamp to x-axis value (days from first data point)
                let timestamp = data.t as f64 / 1000.0; // Convert from milliseconds to seconds
                let price = data.c;
                (timestamp, price)
            })
            .collect();

        // Find min/max values for scaling
        let (min_time, max_time) = data.iter()
            .fold((f64::MAX, f64::MIN), |(min, max), &(time, _)| {
                (min.min(time), max.max(time))
            });

        let (min_price, max_price) = data.iter()
            .fold((f64::MAX, f64::MIN), |(min, max), &(_, price)| {
                (min.min(price), max.max(price))
            });

        // Add some padding to the price range
        let price_padding = (max_price - min_price) * 0.1;
        let price_min = min_price - price_padding;
        let price_max = max_price + price_padding;

        // Create datasets
        let datasets = vec![
            Dataset::default()
                .name("Price")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&data)
        ];

        // Format timestamps for x-axis
        let format_time = |time: &f64| {
            let dt = chrono::NaiveDateTime::from_timestamp_opt(*time as i64, 0).unwrap();
            format!("{}", dt.format("%m/%d"))
        };

        // Format prices for y-axis
        let format_price = |price: &f64| {
            format!("${:.2}", price)
        };

        // Create the chart
        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(
                Axis::default()
                    .title(Line::from(vec![Span::styled("Date", Style::default().fg(Color::White))]))
                    .style(Style::default().fg(Color::White))
                    .bounds([min_time, max_time])
                    .labels(vec![
                        Span::styled(format_time(&min_time), Style::default().fg(Color::White)),
                        Span::styled(format_time(&((min_time + max_time) / 2.0)), Style::default().fg(Color::White)),
                        Span::styled(format_time(&max_time), Style::default().fg(Color::White)),
                    ])
            )
            .y_axis(
                Axis::default()
                    .title(Line::from(vec![Span::styled("Price", Style::default().fg(Color::White))]))
                    .style(Style::default().fg(Color::White))
                    .bounds([price_min, price_max])
                    .labels(vec![
                        Span::styled(format_price(&price_min), Style::default().fg(Color::White)),
                        Span::styled(format_price(&((price_min + price_max) / 2.0)), Style::default().fg(Color::White)),
                        Span::styled(format_price(&price_max), Style::default().fg(Color::White)),
                    ])
            );

        f.render_widget(chart, area);
    } else {
        // Show loading or no data message
        let loading_text = Line::from(vec![
            Span::styled("Loading historical data...", Style::default().fg(Color::Yellow))
        ]);
        let paragraph = ratatui::widgets::Paragraph::new(loading_text)
            .block(block);
        f.render_widget(paragraph, area);
    }
}

pub fn draw_candlestick(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("OHLC Chart: {}", app.symbol))
        .borders(Borders::ALL);

    if let Some(historical_data) = &app.historical_data {
        if historical_data.results.is_empty() {
            let no_data_text = Line::from(vec![
                Span::styled("No historical data available", Style::default().fg(Color::Red))
            ]);
            let paragraph = ratatui::widgets::Paragraph::new(no_data_text)
                .block(block);
            f.render_widget(paragraph, area);
            return;
        }

        // For a real candlestick chart, we'd need a more complex rendering approach
        // This is a simplified version using text representation

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Min(3),     // Data
            ].as_ref())
            .split(area);

        // Render block around the whole area
        f.render_widget(block, area);

        // Render header
        let header = vec![
            Span::styled("Date", Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Open", Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("High", Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Low", Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Close", Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Volume", Style::default().fg(Color::White)),
        ];

        let header_text = ratatui::widgets::Paragraph::new(Line::from(header))
            .style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(header_text, chunks[0]);

        // Render data rows
        let mut rows = Vec::new();
        for data in historical_data.results.iter().take(10) { // Limit to 10 rows for simplicity
            let dt = chrono::NaiveDateTime::from_timestamp_opt((data.t / 1000) as i64, 0).unwrap();
            let date = dt.format("%Y-%m-%d").to_string();

            // Determine if price went up or down
            let color = if data.c >= data.o { Color::Green } else { Color::Red };

            let row = vec![
                Span::styled(date, Style::default().fg(Color::White)),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}", data.o), Style::default().fg(color)),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}", data.h), Style::default().fg(color)),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}", data.l), Style::default().fg(color)),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled(format!("{:.2}", data.c), Style::default().fg(color)),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled(format!("{}", data.v), Style::default().fg(Color::White)),
            ];

            rows.push(Line::from(row));
        }

        let data_text = ratatui::widgets::Paragraph::new(rows)
            .scroll((0, 0));
        f.render_widget(data_text, chunks[1]);
    } else {
        // Show loading or no data message
        let loading_text = Line::from(vec![
            Span::styled("Loading historical data...", Style::default().fg(Color::Yellow))
        ]);
        let paragraph = ratatui::widgets::Paragraph::new(loading_text)
            .block(block);
        f.render_widget(paragraph, area);
    }
}
