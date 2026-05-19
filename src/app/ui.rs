use crate::app::alerts::draw_alerts;
use crate::app::charts::draw_charts;
use crate::app::layout::{centered_rect, shell_vertical_constraints};
use crate::config::ResolvedLayout;
use crate::app::portfolio::draw_portfolio;
use crate::app::styles::ResolvedTheme;
use crate::app::table_filter::filter_title_suffix;
use crate::app::{App, SettingsEdit, Tab};
use crate::config::MarketProviderKind;
use crate::models::ticker::{ticker_response_matches_symbol_for_session, TickerResponse};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
    Terminal,
};
use std::io;
use std::time::Instant;

/// Stock View status fits on one line at or above this width (Issue #81 / SPEC §36.1).
pub(crate) const STOCK_VIEW_STATUS_SINGLE_LINE_COLS: u16 = 100;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
        let layout = app.layout_for_render();
        let size = f.size();
        // Paint theme background for the whole terminal; otherwise only `fg` is applied and
        // the host's default (often black) stays visible — Light preset looked like dark-on-dark.
        f.render_widget(
            Block::default().style(Style::default().bg(rt.background)),
            size,
        );

        let startup_h = u16::from(app.startup_error.is_some()) * 2;
        let status_rows = status_bar_row_count(app, size.width);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(shell_vertical_constraints(&layout, startup_h, status_rows).as_ref())
            .split(size);

        if layout.show_tab_bar {
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
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("StockTerm")
                        .style(rt.canvas())
                        .border_style(Style::default().fg(rt.border).bg(rt.background)),
                )
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
        }

        if startup_h > 0 {
            draw_startup_config_banner(f, app, chunks[1], rt);
        }

        let body = chunks[2];
        let status_area = chunks[3];

        match app.active_tab {
            Tab::StockView => draw_stock_view(f, app, body, rt, layout),
            Tab::Portfolio => draw_portfolio(f, app, body, rt),
            Tab::Alerts => draw_alerts(f, app, body, rt),
            Tab::Search => draw_search(f, app, body, rt),
            Tab::News => draw_news(f, app, body, rt),
            Tab::Charts => draw_charts(f, app, body, rt, layout),
            Tab::Settings => draw_settings(f, app, body, rt),
        }

        if layout.show_status_bar {
            draw_status_bar(f, app, status_area, rt);
        }

        if app.error_log_overlay_open {
            draw_error_log_overlay(f, app, size, rt);
        }
    })?;
    Ok(())
}

fn draw_startup_config_banner(f: &mut Frame, app: &App, area: Rect, rt: ResolvedTheme) {
    let Some(ref err) = app.startup_error else {
        return;
    };
    let style = rt.startup_banner();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Config error")
        .style(style)
        .border_style(Style::default().fg(rt.foreground).bg(rt.selection));
    let line = Line::from(vec![Span::styled(err.status_line(), style)]);
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn error_log_tab_label(tab: Tab) -> &'static str {
    match tab {
        Tab::StockView => "Stock",
        Tab::Portfolio => "Port",
        Tab::Alerts => "Alerts",
        Tab::Search => "Search",
        Tab::News => "News",
        Tab::Charts => "Charts",
        Tab::Settings => "Sets",
    }
}

fn draw_error_log_overlay(f: &mut Frame, app: &mut App, full: Rect, rt: ResolvedTheme) {
    let popup = centered_rect(full, 78, 70);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Recent errors")
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let footer_h = 2u16;
    let list_h = inner.height.saturating_sub(footer_h);
    let visible = list_h.max(1) as usize;

    // Issue #120 / SPEC §20.15.1 — publish the layout-derived visible-row count
    // for the overlay key handlers in `handlers.rs`. This is layout metadata,
    // not scroll state. Scroll itself is read-only here per Issue #121.
    app.error_log_visible_rows = visible;

    let total = app.error_log.len();
    let max_scroll = total.saturating_sub(visible);
    let scroll = app.error_log_scroll.min(max_scroll);

    let entries: Vec<_> = app.error_log.iter().collect();
    let window: Vec<ListItem> = entries
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|e| {
            let t = e.when.format("%H:%M:%S").to_string();
            let tab_s = error_log_tab_label(e.tab);
            let cat = e.category.as_prefix().trim_end_matches(' ');
            let row = format!("{t} {tab_s:>6} {cat} {}", truncate_visual(&e.line, 96));
            ListItem::new(Line::from(Span::raw(row)))
        })
        .collect();

    let list = List::new(window).style(rt.fg_foreground());
    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_h,
    };
    f.render_widget(list, list_area);

    let footer_y = inner.y + list_h;
    let footer_area = Rect {
        x: inner.x,
        y: footer_y,
        width: inner.width,
        height: footer_h,
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "Esc close · j/↓ k/↑ scroll · PgUp/PgDn",
            rt.fg_muted(),
        ),
    ]))
    .style(rt.canvas());
    f.render_widget(footer, footer_area);
}

fn resolve_quote(app: &App) -> Option<&TickerResponse> {
    app.ticker_data
        .as_ref()
        .filter(|t| {
            ticker_response_matches_symbol_for_session(t, &app.symbol, &app.symbol)
        })
        .or_else(|| app.watchlist_quotes.get(&app.symbol))
}

fn draw_stock_view(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme, layout: ResolvedLayout) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(layout.stock_view_watchlist_pct),
            Constraint::Min(6),
        ])
        .split(area);

    draw_watchlist_table(f, app, chunks[0], rt);
    draw_stock_detail(f, app, chunks[1], rt);
}

fn draw_watchlist_table(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme) {
    let selected_style = Style::default()
        .bg(rt.selection)
        .fg(rt.foreground)
        .add_modifier(Modifier::BOLD);

    if app.watchlist.is_empty() {
        let text = vec![
            Line::from(vec![Span::styled(
                "Watchlist is empty. Type a ticker (A–Z), Enter to fetch, then press ",
                rt.fg_border(),
            )]),
            Line::from(vec![
                Span::styled("w", rt.fg_border()),
                Span::styled(" to add. ", rt.canvas()),
                Span::styled("j", rt.fg_border()),
                Span::styled("/", rt.fg_border()),
                Span::styled("k", rt.fg_border()),
                Span::styled(" or arrows move rows; ", rt.canvas()),
                Span::styled("/", rt.fg_border()),
                Span::styled(" filters symbols (Portfolio tab too).", rt.canvas()),
            ]),
        ];
        let block = Block::default()
            .title("Watchlist")
            .borders(Borders::ALL)
            .style(rt.canvas())
            .border_style(Style::default().fg(rt.border).bg(rt.background));
        f.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let wl_title = format!(
        "Watchlist (w add, x/D remove, j/k navigate){}",
        filter_title_suffix(&app.filter_query)
    );

    let filtered_idx = app.watchlist_filter_indices();
    if filtered_idx.is_empty() {
        let text = vec![Line::from(vec![Span::styled(
            "No symbols match filter — Esc clears filter.",
            rt.fg_border(),
        )])];
        let block = Block::default()
            .title(wl_title)
            .borders(Borders::ALL)
            .style(rt.canvas())
            .border_style(Style::default().fg(rt.border).bg(rt.background));
        f.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_cells = ["Symbol", "Last", "Change", "%Chg", "Volume"]
        .iter()
        .map(|h| Cell::from(*h).style(rt.fg_foreground()));

    let header = Row::new(header_cells)
        .style(rt.canvas().add_modifier(Modifier::BOLD))
        .height(1);

    let rows = filtered_idx.iter().map(|&idx| {
        let sym = &app.watchlist[idx];
        let row_style = rt.canvas();
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
                        rt.positive
                    } else {
                        rt.negative
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
                    rt.muted,
                ),
            };

        let cells = [
            Cell::from(sym.as_str()),
            Cell::from(last_s),
            Cell::from(chg_s).style(rt.fg_color(chg_color)),
            Cell::from(pct_s).style(rt.fg_color(chg_color)),
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
            .title(wl_title)
            .style(rt.canvas())
            .border_style(Style::default().fg(rt.border).bg(rt.background)),
    )
    .highlight_style(selected_style)
    .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.watchlist_state);
}

fn draw_stock_detail(f: &mut Frame, app: &App, area: Rect, rt: ResolvedTheme) {
    let block = Block::default()
        .title(format!("Detail: {}", app.symbol))
        .borders(Borders::ALL)
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));

    if let Some(ticker_data) = resolve_quote(app) {
        if ticker_data.results.is_empty() {
            let text = vec![Line::from(vec![Span::styled(
                "No quote data returned for this symbol.",
                rt.fg_border(),
            )])];
            f.render_widget(Paragraph::new(text).block(block), area);
            return;
        }

        let Some(result) = ticker_data.latest_result() else {
            let text = vec![Line::from(vec![Span::styled(
                "No quote data returned for this symbol.",
                rt.fg_border(),
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
            rt.positive
        } else {
            rt.negative
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Symbol: ", rt.canvas()),
                Span::styled(&app.symbol, rt.fg_accent()),
            ]),
            Line::from(vec![
                Span::styled("Price: ", rt.canvas()),
                Span::styled(
                    format!("${:.2}", result.c),
                    rt.fg_foreground(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Change: ", rt.canvas()),
                Span::styled(
                    format!(
                        "{}{:.2} ({:.2}%)",
                        if price_change >= 0.0 { "+" } else { "" },
                        price_change,
                        percent_change
                    ),
                    rt.fg_color(change_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Open: ", rt.canvas()),
                Span::styled(
                    format!("${:.2}", result.o),
                    rt.fg_foreground(),
                ),
            ]),
            Line::from(vec![
                Span::styled("High: ", rt.canvas()),
                Span::styled(
                    format!("${:.2}", result.h),
                    rt.fg_foreground(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Low: ", rt.canvas()),
                Span::styled(
                    format!("${:.2}", result.l),
                    rt.fg_foreground(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Volume: ", rt.canvas()),
                Span::styled(
                    format!("{:.0}", result.v),
                    rt.fg_foreground(),
                ),
            ]),
        ];

        f.render_widget(Paragraph::new(text).block(block), area);
    } else if let Some(error) = app.error_message().as_deref() {
        let text = vec![Line::from(vec![Span::styled(
            error,
            rt.error_text(),
        )])];
        f.render_widget(Paragraph::new(text).block(block), area);
    } else {
        let text = vec![Line::from(vec![Span::styled(
            "Loading...",
            rt.fg_border(),
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

fn draw_search(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(area);

    let query_line = Line::from(vec![
        Span::styled("Query: ", rt.canvas()),
        Span::styled(
            if app.search_query.is_empty() {
                "(type to search)"
            } else {
                app.search_query.as_str()
            },
            rt.fg_accent(),
        ),
    ]);
    let q_block = Block::default()
        .borders(Borders::ALL)
        .title("Symbol search (Esc clear · Enter pick)")
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    f.render_widget(Paragraph::new(query_line).block(q_block), chunks[0]);

    let selected_style = Style::default()
        .bg(rt.selection)
        .fg(rt.foreground)
        .add_modifier(Modifier::BOLD);
    let n = app.search_results_len();
    if n == 0 {
        let msg = if app.search_refresh_inflight {
            Line::from(vec![Span::styled(
                "Searching…",
                rt.fg_border(),
            )])
        } else if app.search_query.trim().is_empty() {
            Line::from(vec![Span::styled(
                "Enter a company or ticker fragment.",
                rt.fg_muted(),
            )])
        } else if app.search_results.is_some() {
            Line::from(vec![Span::styled(
                "No results",
                rt.fg_border(),
            )])
        } else {
            Line::from(vec![Span::styled(
                "Waiting for debounce…",
                rt.fg_muted(),
            )])
        };
        f.render_widget(
            Paragraph::new(msg).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .style(rt.canvas())
                    .border_style(Style::default().fg(rt.border).bg(rt.background)),
            ),
            chunks[1],
        );
    } else {
        let header_cells = ["Symbol", "Name", "Type", "Exchange"]
            .iter()
            .map(|h| Cell::from(*h).style(rt.fg_foreground()));
        let header = Row::new(header_cells)
            .style(rt.canvas().add_modifier(Modifier::BOLD))
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
                .style(rt.canvas())
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results (j/k · Enter)")
                .style(rt.canvas())
                .border_style(Style::default().fg(rt.border).bg(rt.background)),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");
        f.render_stateful_widget(table, chunks[1], &mut app.search_table_state);
    }

    let footer = if app.search_refresh_inflight {
        Line::from(vec![Span::styled(
            "Searching…",
            rt.fg_accent(),
        )])
    } else {
        Line::from(vec![Span::styled(
            "250 ms debounce · one request at a time",
            rt.fg_muted(),
        )])
    };
    f.render_widget(Paragraph::new(footer).style(rt.canvas()), chunks[2]);
}

fn draw_news(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme) {
    let title = format!("News — {}", app.symbol);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title.as_str())
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));

    if app.symbol.is_empty() {
        let t = Line::from(vec![Span::styled(
            "Select a symbol on Stock View first.",
            rt.fg_border(),
        )]);
        f.render_widget(Paragraph::new(t).block(block), area);
        return;
    }

    if app.news_refresh_inflight && app.news_data.is_none() {
        let t = Line::from(vec![Span::styled(
            "Loading…",
            rt.fg_border(),
        )]);
        f.render_widget(Paragraph::new(t).block(block), area);
        return;
    }

    if let Some(data) = &app.news_data {
        if data.results.is_empty() {
            let t = Line::from(vec![Span::styled(
                "No news available",
                rt.fg_border(),
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
                    Span::styled(format!("{pub_name:<20} "), rt.fg_accent()),
                    Span::styled(title_s, rt.canvas()),
                    Span::styled(
                        format!("  [{date_s}]"),
                        rt.fg_muted(),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                block.title(format!(
                    "{} (j/k · Enter open · c copy)",
                    title
                )),
            )
            .highlight_style(
                Style::default()
                    .bg(rt.selection)
                    .fg(rt.foreground)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut app.news_list_state);
        return;
    }

    let t = Line::from(vec![Span::styled(
        "Loading…",
        rt.fg_border(),
    )]);
    f.render_widget(Paragraph::new(t).block(block), area);
}

fn layout_row_summary(app: &App) -> String {
    let saved = app.config.layout.saved_summary_label();

    if app.settings_row == 6 && app.settings_editing.is_none() {
        let preview = {
            let mut l = app.config.layout.clone();
            l.preset = Some(app.settings_layout_draft);
            l.saved_summary_label()
        };
        format!("Preview: {preview} · h/l or ←/→ · Enter save · saved: {saved}")
    } else {
        saved
    }
}

fn theme_row_summary(app: &App) -> String {
    let saved = app
        .config
        .theme
        .as_ref()
        .map(|t| {
            let p = t.effective_preset();
            if t.overrides.is_empty() && t.accent_hex.is_none() && t.background_hex.is_none() {
                format!("preset {}", p.label())
            } else {
                format!("{} + overrides", p.label())
            }
        })
        .unwrap_or_else(|| "default (no theme in file)".to_string());

    if app.settings_row == 3 && app.settings_editing.is_none() {
        format!(
            "Preview: {} · h/l or ←/→ · Enter save · saved: {saved}",
            app.settings_theme_draft.label()
        )
    } else {
        saved
    }
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Settings (j/k · Enter edit/toggle/theme save · Esc cancel)")
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let provider_s = match app.config.provider {
        MarketProviderKind::Yahoo => "yahoo",
        MarketProviderKind::Polygon => "polygon",
    };
    let theme_s = theme_row_summary(app);

    let flash = app
        .settings_saved_flash_until
        .is_some_and(|t| Instant::now() < t);

    let row_style = |i: usize| {
        if i == app.settings_row && app.settings_editing.is_none() {
            Style::default()
                .bg(rt.selection)
                .fg(rt.foreground)
                .add_modifier(Modifier::BOLD)
        } else {
            rt.canvas()
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
        Span::styled(rr_display, rt.canvas()),
        Span::styled(
            "  (effective ≥ 5s; 0 → 30s default)",
            rt.fg_muted(),
        ),
    ]));

    let ds_display = if app.settings_editing == Some(SettingsEdit::DefaultSymbol) {
        format!("> {}_", app.settings_edit_buffer)
    } else {
        app.config.default_symbol.clone()
    };
    lines.push(Line::from(vec![
        Span::styled("1. Default symbol: ", row_style(1)),
        Span::styled(ds_display, rt.canvas()),
    ]));

    let notify_s = if app.config.notifications_enabled { "on" } else { "off" };
    lines.push(Line::from(vec![
        Span::styled("2. Desktop alert toasts: ", row_style(2)),
        Span::styled(notify_s, rt.canvas()),
        Span::styled("  (Enter toggles)", rt.fg_muted()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("3. Theme: ", row_style(3)),
        Span::styled(theme_s, rt.canvas()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("4. Provider (read-only): ", row_style(4)),
        Span::styled(provider_s, rt.canvas()),
    ]));
    let keymap_count = app.config.keymap.as_ref().map_or(0, |m| m.len());
    let keymap_value = if keymap_count == 0 {
        "defaults (README Keymap)".to_string()
    } else {
        format!("{keymap_count} override(s) — README Keymap")
    };
    lines.push(Line::from(vec![
        Span::styled("5. Keymap: ", row_style(5)),
        Span::styled(keymap_value, rt.canvas()),
        Span::styled("  (~/.stockterm.json)", rt.fg_muted()),
    ]));
    let layout_s = layout_row_summary(app);
    lines.push(Line::from(vec![
        Span::styled("6. Layout: ", row_style(6)),
        Span::styled(layout_s, rt.canvas()),
    ]));

    if let Some(e) = &app.settings_inline_error {
        lines.push(Line::from(vec![Span::styled(
            e.as_str(),
            rt.error_text(),
        )]));
    }
    if flash {
        lines.push(Line::from(vec![Span::styled(
            "Saved",
            rt.success_text(),
        )]));
    }

    let p = Paragraph::new(lines).style(rt.canvas());
    f.render_widget(p, inner);
}

/// True when Stock View shows the default hint lines (not error / inflight overrides).
fn stock_view_status_is_hint_mode(app: &App) -> bool {
    app.error_message().is_none()
        && !app.stock_refresh_inflight
        && app.news_url_flash_line().is_none()
}

/// Status shell rows: `0` when hidden; `2` on narrow Stock View hint mode (Issue #81).
pub(crate) fn status_bar_row_count(app: &App, term_width: u16) -> u16 {
    let layout = app.layout_for_render();
    if !layout.show_status_bar {
        return 0;
    }
    if app.active_tab == Tab::StockView
        && term_width < STOCK_VIEW_STATUS_SINGLE_LINE_COLS
        && stock_view_status_is_hint_mode(app)
    {
        2
    } else {
        1
    }
}

fn status_bar_global_suffix(rt: ResolvedTheme) -> Vec<Span<'static>> {
    vec![
        Span::styled(" · ", rt.canvas()),
        Span::styled("^E", rt.fg_border()),
        Span::styled(" error log · ", rt.canvas()),
        Span::styled("^R", rt.fg_border()),
        Span::styled(" retry", rt.canvas()),
    ]
}

fn stock_view_status_primary_spans(rt: ResolvedTheme) -> Vec<Span<'static>> {
    vec![
        Span::styled("q quit · Tab tabs · ", rt.canvas()),
        Span::styled("A–Z", rt.fg_border()),
        Span::styled(" type · ", rt.canvas()),
        Span::styled("w", rt.fg_border()),
        Span::styled(" add · ", rt.canvas()),
        Span::styled("x", rt.fg_border()),
        Span::styled("/", rt.canvas()),
        Span::styled("D", rt.fg_border()),
        Span::styled(" rm · ", rt.canvas()),
        Span::styled("j/k", rt.fg_border()),
        Span::styled(" · Enter", rt.canvas()),
    ]
}

/// Stock View status hint lines for `width` (one or two lines per SPEC §36.1.3).
pub(crate) fn stock_view_status_lines(width: u16, rt: ResolvedTheme) -> Vec<Line<'static>> {
    let mut primary = stock_view_status_primary_spans(rt);
    primary.extend(status_bar_global_suffix(rt));

    if width >= STOCK_VIEW_STATUS_SINGLE_LINE_COLS {
        let mut line = Line::from(primary);
        line.spans.push(Span::styled(" · ", rt.canvas()));
        line.spans.push(Span::styled(
            "tickers w/x/j/k: Shift+1st letter if lower",
            rt.fg_muted(),
        ));
        vec![line]
    } else {
        let line1 = Line::from(primary);
        let line2 = Line::from(vec![Span::styled(
            "Symbols starting w/x/j/k: type 1st letter with Shift (e.g. Wmt → WMT)",
            rt.fg_muted(),
        )]);
        vec![line1, line2]
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, rt: ResolvedTheme) {
    let lines: Vec<Line> = if let Some(error) = app.error_message() {
        vec![Line::from(vec![Span::styled(error, rt.error_text())])]
    } else if app.active_tab == Tab::Search && app.search_refresh_inflight {
        vec![Line::from(vec![Span::styled(
            "Searching…",
            rt.fg_accent(),
        )])]
    } else if let Some(flash) = app.news_url_flash_line() {
        vec![Line::from(vec![Span::styled(flash, rt.success_text())])]
    } else if app.active_tab == Tab::News && app.news_refresh_inflight {
        vec![Line::from(vec![Span::styled(
            "Loading news…",
            rt.fg_accent(),
        )])]
    } else if app.stock_refresh_inflight {
        vec![Line::from(vec![Span::styled(
            "Refreshing quotes…",
            rt.fg_accent(),
        )])]
    } else if app.active_tab == Tab::StockView && stock_view_status_is_hint_mode(app) {
        stock_view_status_lines(area.width.max(1), rt)
    } else {
        let mut line = match app.active_tab {
            Tab::Search => Line::from(vec![
                Span::styled("q quit · Tab tabs · type query · ", rt.canvas()),
                Span::styled("Esc", rt.fg_border()),
                Span::styled(" clear · ", rt.canvas()),
                Span::styled("Enter", rt.fg_border()),
                Span::styled(" open stock", rt.canvas()),
            ]),
            Tab::News => Line::from(vec![
                Span::styled("q quit · Tab tabs · ", rt.canvas()),
                Span::styled("Enter", rt.fg_border()),
                Span::styled(" open · ", rt.canvas()),
                Span::styled("c", rt.fg_border()),
                Span::styled(" copy · ", rt.canvas()),
                Span::styled("j/k", rt.fg_border()),
                Span::styled(" scroll", rt.canvas()),
            ]),
            Tab::Settings => Line::from(vec![
                Span::styled("q quit · Tab tabs · ", rt.canvas()),
                Span::styled("Enter", rt.fg_border()),
                Span::styled(" edit row · ", rt.canvas()),
                Span::styled("Esc", rt.fg_border()),
                Span::styled(" cancel edit", rt.canvas()),
            ]),
            _ => Line::from(vec![Span::styled(
                "q quit · Tab tabs · see Stock View for watchlist keys",
                rt.canvas(),
            )]),
        };
        line.spans.extend(status_bar_global_suffix(rt));
        vec![line]
    };

    let paragraph = Paragraph::new(lines).style(rt.canvas());
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod status_bar_tests {
    use super::*;
    use crate::app::App;
    use crate::config::theme::{Theme, ThemePreset};

    fn test_rt() -> ResolvedTheme {
        ResolvedTheme::from_palette(Theme::from_preset(ThemePreset::BuiltinDefault).resolve_rgb())
    }

    #[test]
    fn stock_view_status_lines_wide_is_one_line() {
        let lines = stock_view_status_lines(120, test_rt());
        assert_eq!(lines.len(), 1);
        let text: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("Shift+1st letter"));
    }

    #[test]
    fn stock_view_status_lines_narrow_is_two_lines() {
        let lines = stock_view_status_lines(80, test_rt());
        assert_eq!(lines.len(), 2);
        let text: String = lines[1]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("Shift"));
    }

    #[test]
    fn status_bar_row_count_stock_view_narrow_is_two() {
        let mut app = App::new();
        app.active_tab = Tab::StockView;
        app.active_runtime_error = None;
        app.stock_refresh_inflight = false;
        assert_eq!(status_bar_row_count(&app, 80), 2);
    }

    #[test]
    fn status_bar_row_count_inflight_is_one() {
        let mut app = App::new();
        app.stock_refresh_inflight = true;
        assert_eq!(status_bar_row_count(&app, 80), 1);
    }
}
