use crate::app::alerts::draw_alerts;
use crate::app::charts::draw_charts;
use crate::app::portfolio::draw_portfolio;
use crate::app::{App, Tab};
use crate::models::ticker::TickerResponse;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
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

fn resolve_quote(app: &App) -> Option<&TickerResponse> {
    app.ticker_data
        .as_ref()
        .filter(|t| t.ticker.is_empty() || t.ticker.eq_ignore_ascii_case(&app.symbol))
        .or_else(|| app.watchlist_quotes.get(&app.symbol))
}

fn draw_stock_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Min(6),
        ])
        .split(area);

    draw_watchlist_table(f, app, chunks[0]);
    draw_stock_detail(f, app, chunks[1]);
}

fn draw_watchlist_table(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    if app.watchlist.is_empty() {
        let text = vec![
            Line::from(vec![Span::styled(
                "Watchlist is empty. Type a ticker (A–Z), Enter to fetch, then press ",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![
                Span::styled("w", Style::default().fg(Color::Yellow)),
                Span::raw(" to add it. "),
                Span::styled("j", Style::default().fg(Color::Yellow)),
                Span::raw("/"),
                Span::styled("k", Style::default().fg(Color::Yellow)),
                Span::raw(" or arrows move selection."),
            ]),
        ];
        let block = Block::default()
            .title("Watchlist")
            .borders(Borders::ALL);
        f.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_cells = ["Symbol", "Last", "Change", "%Chg", "Volume"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = app.watchlist.iter().map(|sym| {
        let row_style = Style::default();
        let (last_s, chg_s, pct_s, vol_s, chg_color) =
            match app.watchlist_quotes.get(sym).and_then(|r| r.latest_result()) {
                Some(bar) => {
                    let price_change = bar.c - bar.o;
                    let pct = if bar.o.abs() > f64::EPSILON {
                        (price_change / bar.o) * 100.0
                    } else {
                        0.0
                    };
                    let chg_color = if price_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    (
                        format!("${:.2}", bar.c),
                        format!(
                            "{}{:.2}",
                            if price_change >= 0.0 { "+" } else { "" },
                            price_change
                        ),
                        format!(
                            "{}{:.2}%",
                            if price_change >= 0.0 { "+" } else { "" },
                            pct
                        ),
                        format!("{:.0}", bar.v),
                        chg_color,
                    )
                }
                _ => (
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    Color::DarkGray,
                ),
            };

        let cells = [
            Cell::from(sym.as_str()),
            Cell::from(last_s),
            Cell::from(chg_s).style(Style::default().fg(chg_color)),
            Cell::from(pct_s).style(Style::default().fg(chg_color)),
            Cell::from(vol_s),
        ];
        Row::new(cells).height(1).style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Min(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Watchlist (w add, x/D remove, j/k navigate)"),
    )
    .highlight_style(selected_style)
    .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.watchlist_state);
}

fn draw_stock_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Detail: {}", app.symbol))
        .borders(Borders::ALL);

    if let Some(ticker_data) = resolve_quote(app) {
        if ticker_data.results.is_empty() {
            let text = vec![Line::from(vec![Span::styled(
                "No quote data returned for this symbol.",
                Style::default().fg(Color::Yellow),
            )])];
            f.render_widget(Paragraph::new(text).block(block), area);
            return;
        }

        let Some(result) = ticker_data.latest_result() else {
            let text = vec![Line::from(vec![Span::styled(
                "No quote data returned for this symbol.",
                Style::default().fg(Color::Yellow),
            )])];
            f.render_widget(Paragraph::new(text).block(block), area);
            return;
        };
        let price_change = result.c - result.o;
        let percent_change = if result.o.abs() > f64::EPSILON {
            (price_change / result.o) * 100.0
        } else {
            0.0
        };
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

        f.render_widget(Paragraph::new(text).block(block), area);
    } else if let Some(error) = &app.error_message {
        let text = vec![Line::from(vec![Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        )])];
        f.render_widget(Paragraph::new(text).block(block), area);
    } else {
        let text = vec![Line::from(vec![Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )])];
        f.render_widget(Paragraph::new(text).block(block), area);
    }
}

fn draw_search(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_news(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_settings(_f: &mut Frame, _app: &App, _area: Rect) {}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(error) = &app.error_message {
        Line::from(vec![Span::styled(error.as_str(), Style::default().fg(Color::Red))])
    } else if app.stock_refresh_inflight {
        Line::from(vec![Span::styled(
            "Refreshing quotes…",
            Style::default().fg(Color::Cyan),
        )])
    } else {
        Line::from(vec![
            Span::raw("q quit · Tab tabs · "),
            Span::styled("w", Style::default().fg(Color::Yellow)),
            Span::raw(" add · "),
            Span::styled("x", Style::default().fg(Color::Yellow)),
            Span::raw("/"),
            Span::styled("D", Style::default().fg(Color::Yellow)),
            Span::raw(" remove · "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" move · Enter fetch"),
        ])
    };

    let paragraph = Paragraph::new(status);
    f.render_widget(paragraph, area);
}
