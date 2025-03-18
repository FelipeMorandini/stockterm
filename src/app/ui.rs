use crate::app::App;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        let block = Block::default()
            .title(format!("Stock Info: {}", app.symbol))
            .borders(Borders::ALL);

        let text = if let Some(ticker_data) = &app.ticker_data {
            let ticker_info = format!("{:#?}", ticker_data.results[0]);
            Text::from(vec![
                Span::raw(ticker_info),
            ])
        } else {
            Text::from(Span::raw("Loading..."))
        };

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, chunks[0]);
    })?;
    Ok(())
}