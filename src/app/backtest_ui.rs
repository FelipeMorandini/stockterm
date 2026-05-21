//! Backtest tab drawing (Issue #25 / SPEC §47.4).

use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::models::backtest::BacktestStrategyKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table};
use ratatui::Frame;

/// Preformatted left-pane lines (`false` = primary, `true` = muted). Built outside `draw`.
pub(crate) struct BacktestParamsCache {
    pub lines: Vec<(String, bool)>,
    pub summary_placeholder_rows: Vec<Row<'static>>,
}

/// Precomputed table/chart data for the Backtest tab (built after `FetchDone`, not in `draw`).
pub(crate) struct BacktestDrawCache {
    pub summary_table_rows: Vec<Row<'static>>,
    pub equity_points: Vec<(f64, f64)>,
    pub trade_table_rows: Vec<Row<'static>>,
}

/// Clears session backtest output (stale after symbol/range change or chart refetch).
pub(crate) fn clear_backtest_session(app: &mut App) {
    app.backtest_report = None;
    app.backtest_draw_cache = None;
    app.backtest_trade_list_state.select(None);
}

/// Rebuilds [`App::backtest_params_cache`] from symbol, config, and chart data.
pub(crate) fn rebuild_backtest_params_cache(app: &mut App) {
    let p = &app.config.backtest_strategy;
    let sim = &app.config.backtest;
    let strategy_label = match p.kind {
        BacktestStrategyKind::SmaCrossover => "SMA crossover",
        BacktestStrategyKind::RsiMeanReversion => "RSI mean-reversion",
    };
    let bar_hint = app
        .historical_data
        .as_ref()
        .map(|h| format!("Bars loaded: {} (Charts range)", h.results.len()))
        .unwrap_or_else(|| "No chart data — open Charts tab first".to_string());

    let lines = vec![
        (format!("Symbol: {}", app.symbol), false),
        (bar_hint, true),
        (String::new(), false),
        (format!("Strategy: {strategy_label}"), false),
        (
            format!("  SMA fast/slow: {} / {}", p.sma_fast, p.sma_slow),
            true,
        ),
        (
            format!(
                "  RSI {} ({} / {})",
                p.rsi_period, p.rsi_oversold, p.rsi_overbought
            ),
            true,
        ),
        (String::new(), false),
        ("Simulation".into(), false),
        (
            format!("  Capital: ${:.2}", sim.initial_capital),
            true,
        ),
        (
            format!("  Commission/trade: ${:.2}", sim.commission_per_trade),
            true,
        ),
        (
            format!("  Slippage: {:.1} bps", sim.slippage_bps),
            true,
        ),
        (String::new(), false),
        (
            "Edit capital/fees: Settings rows 7–9".into(),
            true,
        ),
        (
            "Long-only · close fills · force-flat last bar".into(),
            true,
        ),
    ];

    app.backtest_params_cache = Some(BacktestParamsCache {
        lines,
        summary_placeholder_rows: vec![Row::new(vec![
            Cell::from("—"),
            Cell::from("Run backtest (Enter)"),
        ])],
    });
}

/// Rebuilds [`App::backtest_draw_cache`] from [`App::backtest_report`].
pub(crate) fn rebuild_backtest_draw_cache(app: &mut App) {
    let Some(report) = &app.backtest_report else {
        app.backtest_draw_cache = None;
        return;
    };
    let s = &report.summary;
    let summary_table_rows = vec![
        Row::new(vec![Cell::from("PnL"), Cell::from(format!("${:.2}", s.total_pnl))]),
        Row::new(vec![
            Cell::from("Return %"),
            Cell::from(format!("{:.2}%", s.total_return_pct)),
        ]),
        Row::new(vec![
            Cell::from("Max DD %"),
            Cell::from(format!("{:.2}%", s.max_drawdown_pct)),
        ]),
        Row::new(vec![
            Cell::from("Win rate"),
            Cell::from(format!("{:.1}%", s.win_rate_pct)),
        ]),
        Row::new(vec![
            Cell::from("Sharpe"),
            Cell::from(format!("{:.3}", s.sharpe)),
        ]),
        Row::new(vec![
            Cell::from("Trades"),
            Cell::from(s.trade_count.to_string()),
        ]),
    ];
    let equity_points: Vec<(f64, f64)> = report
        .equity_curve
        .iter()
        .map(|(t, eq)| (*t as f64 / 1000.0, *eq))
        .collect();
    let trade_table_rows: Vec<Row<'static>> = report
        .trades
        .iter()
        .map(|t| {
            Row::new(vec![
                Cell::from(t.entry_ts.to_string()),
                Cell::from(t.exit_ts.to_string()),
                Cell::from(format!("{:.2}", t.pnl)),
            ])
        })
        .collect();
    app.backtest_draw_cache = Some(BacktestDrawCache {
        summary_table_rows,
        equity_points,
        trade_table_rows,
    });
}

/// Renders the Backtest tab (params, summary, equity curve, trades).
pub fn draw_backtest(f: &mut Frame, app: &mut App, area: Rect, theme: ResolvedTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Backtest (Enter/r run · n strategy · x export · j/k trades)")
        .style(theme.canvas())
        .border_style(Style::default().fg(theme.border).bg(theme.background));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    draw_backtest_left(f, app, chunks[0], theme);
    draw_backtest_right(f, app, chunks[1], theme);
}

fn draw_backtest_left(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Parameters")
        .style(theme.canvas());

    let Some(cache) = &app.backtest_params_cache else {
        f.render_widget(
            Paragraph::new("Loading…").block(block),
            area,
        );
        return;
    };

    let lines: Vec<Line> = cache
        .lines
        .iter()
        .map(|(text, muted)| {
            if text.is_empty() {
                return Line::from("");
            }
            let style = if *muted {
                theme.fg_muted()
            } else {
                theme.canvas()
            };
            Line::from(Span::styled(text.as_str(), style))
        })
        .collect();

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_backtest_right(f: &mut Frame, app: &mut App, area: Rect, theme: ResolvedTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(10),
        ])
        .split(area);

    draw_backtest_summary(f, app, chunks[0], theme);
    draw_backtest_equity(f, app, chunks[1], theme);
    draw_backtest_trades(f, app, chunks[2], theme);
}

fn draw_backtest_summary(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let header = Row::new(vec![
        Cell::from("Metric"),
        Cell::from("Value"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row<'_>> = if let Some(cache) = &app.backtest_draw_cache {
        cache.summary_table_rows.to_vec()
    } else if let Some(params) = &app.backtest_params_cache {
        params.summary_placeholder_rows.to_vec()
    } else {
        return;
    };

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Summary")
                .style(theme.canvas()),
        );
    f.render_widget(table, area);
}

fn draw_backtest_equity(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Equity curve")
        .style(theme.canvas());

    let Some(cache) = &app.backtest_draw_cache else {
        f.render_widget(
            Paragraph::new("No equity data yet").block(block),
            area,
        );
        return;
    };

    if cache.equity_points.is_empty() {
        f.render_widget(
            Paragraph::new("Empty equity curve").block(block),
            area,
        );
        return;
    }

    let points = &cache.equity_points;
    let y_min = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let y_max = points.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);
    let pad = ((y_max - y_min) * 0.05).max(1.0);

    let dataset = Dataset::default()
        .name("Equity")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.foreground))
        .data(points);

    let chart = Chart::new(vec![dataset])
        .block(block)
        .x_axis(
            ratatui::widgets::Axis::default().bounds([
                points.first().map(|p| p.0).unwrap_or(0.0),
                points.last().map(|p| p.0).unwrap_or(1.0),
            ]),
        )
        .y_axis(
            ratatui::widgets::Axis::default().bounds([y_min - pad, y_max + pad]),
        );

    f.render_widget(chart, area);
}

fn draw_backtest_trades(f: &mut Frame, app: &mut App, area: Rect, theme: ResolvedTheme) {
    let header = Row::new(vec![
        Cell::from("Entry"),
        Cell::from("Exit"),
        Cell::from("PnL"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row<'_>> = app
        .backtest_draw_cache
        .as_ref()
        .map(|c| c.trade_table_rows.to_vec())
        .unwrap_or_default();

    let table = Table::new(rows, [
        Constraint::Percentage(35),
        Constraint::Percentage(35),
        Constraint::Percentage(30),
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Trades")
            .style(theme.canvas()),
    )
    .highlight_style(Style::default().bg(theme.selection).fg(theme.foreground));

    f.render_stateful_widget(table, area, &mut app.backtest_trade_list_state);
}
