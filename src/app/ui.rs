use crate::app::alerts::draw_alerts;
use crate::app::charts::draw_charts;
use crate::app::portfolio::draw_portfolio;
use crate::app::{App, SettingsEdit, Tab};
use crate::config::MarketProviderKind;
use crate::models::ticker::TickerResponse;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
    Terminal,
};
use std::io;
use std::time::Instant;

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

fn truncate_visual(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn draw_search(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(area);

    let query_line = Line::from(vec![
        Span::raw("Query: "),
        Span::styled(
            if app.search_query.is_empty() {
                "(type to search)"
            } else {
                app.search_query.as_str()
            },
            Style::default().fg(Color::Cyan),
        ),
    ]);
    let q_block = Block::default()
        .borders(Borders::ALL)
        .title("Symbol search (Esc clear · Enter pick)");
    f.render_widget(Paragraph::new(query_line).block(q_block), chunks[0]);

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let n = app.search_results_len();
    if n == 0 {
        let msg = if app.search_refresh_inflight {
            Line::from(vec![Span::styled(
                "Searching…",
                Style::default().fg(Color::Yellow),
            )])
        } else if app.search_query.trim().is_empty() {
            Line::from(vec![Span::styled(
                "Enter a company or ticker fragment.",
                Style::default().fg(Color::DarkGray),
            )])
        } else if app.search_results.is_some() {
            Line::from(vec![Span::styled(
                "No results",
                Style::default().fg(Color::Yellow),
            )])
        } else {
            Line::from(vec![Span::styled(
                "Waiting for debounce…",
                Style::default().fg(Color::DarkGray),
            )])
        };
        f.render_widget(
            Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Results")),
            chunks[1],
        );
    } else {
        let header_cells = ["Symbol", "Name", "Type", "Exchange"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);
        let rows = app.search_results.iter().flat_map(|r| {
            r.results.iter().map(|row| {
                let ex = if row.primary_exchange.is_empty() {
                    row.market.as_str()
                } else {
                    row.primary_exchange.as_str()
                };
                Row::new(vec![
                    Cell::from(truncate_visual(&row.ticker, 12)),
                    Cell::from(truncate_visual(&row.name, 36)),
                    Cell::from(truncate_visual(&row.type_, 14)),
                    Cell::from(truncate_visual(ex, 12)),
                ])
                .height(1)
            })
        });
        let table = Table::new(
            rows,
            [
                Constraint::Min(6),
                Constraint::Min(20),
                Constraint::Min(10),
                Constraint::Min(8),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Results (j/k · Enter)"))
        .highlight_style(selected_style)
        .highlight_symbol("> ");
        f.render_stateful_widget(table, chunks[1], &mut app.search_table_state);
    }

    let footer = if app.search_refresh_inflight {
        Line::from(vec![Span::styled(
            "Searching…",
            Style::default().fg(Color::Cyan),
        )])
    } else {
        Line::from(vec![Span::styled(
            "250 ms debounce · one request at a time",
            Style::default().fg(Color::DarkGray),
        )])
    };
    f.render_widget(Paragraph::new(footer), chunks[2]);
}

fn draw_news(f: &mut Frame, app: &mut App, area: Rect) {
    let title = format!("News — {}", app.symbol);
    let block = Block::default().borders(Borders::ALL).title(title.as_str());

    if app.symbol.is_empty() {
        let t = Line::from(vec![Span::styled(
            "Select a symbol on Stock View first.",
            Style::default().fg(Color::Yellow),
        )]);
        f.render_widget(Paragraph::new(t).block(block), area);
        return;
    }

    if app.news_refresh_inflight && app.news_data.is_none() {
        let t = Line::from(vec![Span::styled(
            "Loading…",
            Style::default().fg(Color::Yellow),
        )]);
        f.render_widget(Paragraph::new(t).block(block), area);
        return;
    }

    if let Some(data) = &app.news_data {
        if data.results.is_empty() {
            let t = Line::from(vec![Span::styled(
                "No news available",
                Style::default().fg(Color::Yellow),
            )]);
            f.render_widget(Paragraph::new(t).block(block), area);
            return;
        }

        let items: Vec<ListItem> = data
            .results
            .iter()
            .map(|item| {
                let pub_name = truncate_visual(&item.publisher.name, 18);
                let title_s = truncate_visual(&item.title, 52);
                let date_s = truncate_visual(&item.published_utc, 22);
                let line = Line::from(vec![
                    Span::styled(format!("{pub_name:<20} "), Style::default().fg(Color::Cyan)),
                    Span::raw(title_s),
                    Span::styled(
                        format!("  [{date_s}]"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                block.title(format!(
                    "{} (j/k · Enter open)",
                    title
                )),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut app.news_list_state);
        return;
    }

    let t = Line::from(vec![Span::styled(
        "Loading…",
        Style::default().fg(Color::Yellow),
    )]);
    f.render_widget(Paragraph::new(t).block(block), area);
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Settings (j/k · Enter edit · Esc cancel)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let provider_s = match app.config.provider {
        MarketProviderKind::Yahoo => "yahoo",
        MarketProviderKind::Polygon => "polygon",
    };
    let theme_s = app
        .config
        .theme
        .as_ref()
        .map(|t| format!("{t:?}"))
        .unwrap_or_else(|| "Not configured".to_string());

    let flash = app
        .settings_saved_flash_until
        .is_some_and(|t| Instant::now() < t);

    let row_style = |i: usize| {
        if i == app.settings_row && app.settings_editing.is_none() {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        }
    };

    let mut lines: Vec<Line> = Vec::new();

    let rr_display = if app.settings_editing == Some(SettingsEdit::RefreshRate) {
        format!("> {}_", app.settings_edit_buffer)
    } else {
        format!("{}", app.config.refresh_rate)
    };
    lines.push(Line::from(vec![
        Span::styled("0. Refresh (seconds): ", row_style(0)),
        Span::raw(rr_display),
        Span::styled(
            "  (effective minimum poll: 5s)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let ds_display = if app.settings_editing == Some(SettingsEdit::DefaultSymbol) {
        format!("> {}_", app.settings_edit_buffer)
    } else {
        app.config.default_symbol.clone()
    };
    lines.push(Line::from(vec![
        Span::styled("1. Default symbol: ", row_style(1)),
        Span::raw(ds_display),
    ]));

    lines.push(Line::from(vec![
        Span::styled("2. Theme: ", row_style(2)),
        Span::raw(theme_s),
    ]));
    lines.push(Line::from(vec![
        Span::styled("3. Provider (read-only): ", row_style(3)),
        Span::raw(provider_s),
    ]));
    lines.push(Line::from(vec![
        Span::styled("4. Keymap: ", row_style(4)),
        Span::styled("Coming later (#13)", Style::default().fg(Color::DarkGray)),
    ]));

    if let Some(e) = &app.settings_inline_error {
        lines.push(Line::from(vec![Span::styled(
            e.as_str(),
            Style::default().fg(Color::Red),
        )]));
    }
    if flash {
        lines.push(Line::from(vec![Span::styled(
            "Saved",
            Style::default().fg(Color::Green),
        )]));
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(error) = &app.error_message {
        Line::from(vec![Span::styled(error.as_str(), Style::default().fg(Color::Red))])
    } else if app.active_tab == Tab::Search && app.search_refresh_inflight {
        Line::from(vec![Span::styled(
            "Searching…",
            Style::default().fg(Color::Cyan),
        )])
    } else if app.active_tab == Tab::News && app.news_refresh_inflight {
        Line::from(vec![Span::styled(
            "Loading news…",
            Style::default().fg(Color::Cyan),
        )])
    } else if app.stock_refresh_inflight {
        Line::from(vec![Span::styled(
            "Refreshing quotes…",
            Style::default().fg(Color::Cyan),
        )])
    } else {
        match app.active_tab {
            Tab::StockView => Line::from(vec![
                Span::raw("q quit · Tab tabs · "),
                Span::styled("A–Z", Style::default().fg(Color::Yellow)),
                Span::raw(" type · "),
                Span::styled("w", Style::default().fg(Color::Yellow)),
                Span::raw(" add · "),
                Span::styled("x", Style::default().fg(Color::Yellow)),
                Span::raw("/"),
                Span::styled("D", Style::default().fg(Color::Yellow)),
                Span::raw(" rm · "),
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(" · Enter · "),
                Span::styled(
                    "tickers w/x/j/k: Shift+1st letter if lower",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Tab::Search => Line::from(vec![
                Span::raw("q quit · Tab tabs · type query · "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" clear · "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" open stock"),
            ]),
            Tab::News => Line::from(vec![
                Span::raw("q quit · Tab tabs · "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" open URL · "),
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(" scroll"),
            ]),
            Tab::Settings => Line::from(vec![
                Span::raw("q quit · Tab tabs · "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" edit row · "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cancel edit"),
            ]),
            _ => Line::from(vec![Span::raw(
                "q quit · Tab tabs · see Stock View for watchlist keys",
            )]),
        }
    };

    let paragraph = Paragraph::new(status);
    f.render_widget(paragraph, area);
}
