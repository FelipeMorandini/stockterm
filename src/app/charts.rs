//! Charts tab: line chart, candlesticks, viewport, indicators (Issues #7 / #8 / #9, #21).

use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::config::{MarketProviderKind, ResolvedLayout};
use crate::indicators::{
    ema, macd, rsi, sma, MacdOutput, EMA_PERIOD, MACD_FAST, MACD_SIGNAL, MACD_SLOW, RSI_PERIOD,
    SMA_PERIOD,
};
use crate::indicators::types::IndicatorSeries;
use crate::models::historical::{HistoricalData, HistoricalResponse};
use crate::models::time_range::TimeRange;
use chrono::{DateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
    Frame,
};

/// Pan/zoom window over sorted `HistoricalResponse::results` (half-open `start..end`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChartViewport {
    pub start: usize,
    pub end: usize,
}

impl ChartViewport {
    pub fn full(len: usize) -> Self {
        if len == 0 {
            Self { start: 0, end: 0 }
        } else {
            Self { start: 0, end: len }
        }
    }

    fn width(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn normalize(&mut self, len: usize) {
        if len == 0 {
            self.start = 0;
            self.end = 0;
            return;
        }
        self.end = self.end.clamp(1, len);
        self.start = self.start.min(self.end - 1);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartDisplayMode {
    #[default]
    Line,
    Candlestick,
}

impl ChartDisplayMode {
    pub fn toggle(self) -> Self {
        match self {
            ChartDisplayMode::Line => ChartDisplayMode::Candlestick,
            ChartDisplayMode::Candlestick => ChartDisplayMode::Line,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ChartDisplayMode::Line => "line",
            ChartDisplayMode::Candlestick => "candles",
        }
    }
}

/// Session-only indicator toggles (Issue #21 / SPEC §46.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChartIndicatorToggles {
    pub sma_20: bool,
    pub ema_20: bool,
    pub rsi_14: bool,
    pub macd: bool,
}

impl ChartIndicatorToggles {
    /// True when any indicator toggle is enabled.
    pub fn any_enabled(self) -> bool {
        self.sma_20 || self.ema_20 || self.rsi_14 || self.macd
    }

    /// True when RSI and/or MACD need a sub-pane below the price chart.
    pub fn needs_subpane(self) -> bool {
        self.rsi_14 || self.macd
    }

    /// True when SMA/EMA overlays are enabled (line chart only in v1).
    pub fn overlay_enabled(self) -> bool {
        self.sma_20 || self.ema_20
    }
}

/// Precomputed indicator values for the full historical series (re-sliced at draw).
#[derive(Debug, Clone)]
pub struct ChartIndicatorCache {
    pub sma_20: IndicatorSeries,
    pub ema_20: IndicatorSeries,
    pub rsi_14: IndicatorSeries,
    pub macd: MacdOutput,
}

impl ChartIndicatorCache {
    /// Build all default-period indicators from close prices (oldest → newest).
    pub fn from_closes(closes: &[f64]) -> Self {
        Self {
            sma_20: sma(closes, SMA_PERIOD),
            ema_20: ema(closes, EMA_PERIOD),
            rsi_14: rsi(closes, RSI_PERIOD),
            macd: macd(closes, MACD_FAST, MACD_SLOW, MACD_SIGNAL),
        }
    }
}

/// Visible bars for rendering; empty if there is no data.
pub fn visible_slice<'a>(results: &'a [HistoricalData], vp: &ChartViewport) -> &'a [HistoricalData] {
    let len = results.len();
    if len == 0 {
        return &[];
    }
    let end = if vp.end == 0 {
        len
    } else {
        vp.end.min(len).max(1)
    };
    let start = vp.start.min(end - 1);
    &results[start..end]
}

/// Whether the viewport shows the entire series (same rule as “full” in [`visible_slice`]).
fn viewport_covers_full_series(vp: ChartViewport, series_len: usize) -> bool {
    if series_len == 0 {
        return true;
    }
    if vp.end == 0 {
        return vp.start == 0;
    }
    vp.start == 0 && vp.end >= series_len
}

/// Clamp pan/zoom indices when bar count changes (e.g. one new daily bar).
pub fn clamp_viewport_to_len(vp: ChartViewport, len: usize) -> ChartViewport {
    if len == 0 {
        return ChartViewport::default();
    }
    let end = if vp.end == 0 {
        len
    } else {
        vp.end.min(len).max(1)
    };
    let start = vp.start.min(end.saturating_sub(1));
    ChartViewport { start, end }
}

fn effective_series_ticker<'a>(series: &'a HistoricalResponse, requested_symbol: &'a str) -> &'a str {
    let t = series.ticker.trim();
    if t.is_empty() {
        requested_symbol
    } else {
        t
    }
}

/// Preserve zoom/pan across periodic historical refetch; reset on first load, ticker change, or full-range view.
///
/// `requested_symbol` is the ticker passed to the provider (Issues #64 — Yahoo may omit `series.ticker`).
pub fn chart_viewport_after_refresh(
    previous_series: Option<&HistoricalResponse>,
    current_vp: ChartViewport,
    new_data: &HistoricalResponse,
    requested_symbol: &str,
) -> ChartViewport {
    let new_len = new_data.results.len();
    if new_len == 0 {
        return ChartViewport::default();
    }
    let Some(prev) = previous_series else {
        return ChartViewport::full(new_len);
    };
    if !effective_series_ticker(prev, requested_symbol)
        .eq_ignore_ascii_case(effective_series_ticker(new_data, requested_symbol))
    {
        return ChartViewport::full(new_len);
    }
    let old_len = prev.results.len();
    if viewport_covers_full_series(current_vp, old_len) {
        return ChartViewport::full(new_len);
    }
    clamp_viewport_to_len(current_vp, new_len)
}

pub fn viewport_zoom_in(vp: &mut ChartViewport, len: usize) {
    if len < 2 {
        return;
    }
    vp.normalize(len);
    let w = vp.width();
    if w <= 2 {
        return;
    }
    let new_w = (w / 2).max(2);
    let center = (vp.start + vp.end) / 2;
    let mut new_start = center.saturating_sub(new_w / 2);
    let mut new_end = new_start + new_w;
    if new_end > len {
        new_end = len;
        new_start = new_end.saturating_sub(new_w);
    }
    vp.start = new_start;
    vp.end = new_end;
    vp.normalize(len);
}

pub fn viewport_zoom_out(vp: &mut ChartViewport, len: usize) {
    if len == 0 {
        return;
    }
    vp.normalize(len);
    let w = vp.width();
    let center = (vp.start + vp.end) / 2;
    let new_w = (w.saturating_mul(2)).min(len).max(2);
    let mut new_start = center.saturating_sub(new_w / 2);
    let mut new_end = new_start + new_w;
    if new_end > len {
        new_end = len;
        new_start = 0;
    }
    vp.start = new_start;
    vp.end = new_end;
    vp.normalize(len);
}

pub fn viewport_pan_left(vp: &mut ChartViewport, len: usize) {
    if len < 2 || vp.start == 0 {
        return;
    }
    vp.normalize(len);
    vp.start = vp.start.saturating_sub(1);
    vp.end = vp.end.saturating_sub(1);
    vp.normalize(len);
}

pub fn viewport_pan_right(vp: &mut ChartViewport, len: usize) {
    if len < 2 || vp.end >= len {
        return;
    }
    vp.normalize(len);
    vp.start += 1;
    vp.end += 1;
    vp.normalize(len);
}

fn price_bounds(slice: &[HistoricalData]) -> Option<(f64, f64)> {
    if slice.is_empty() {
        return None;
    }
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for b in slice {
        lo = lo.min(b.l).min(b.o).min(b.c);
        hi = hi.max(b.h).max(b.o).max(b.c);
    }
    if !lo.is_finite() || !hi.is_finite() {
        return None;
    }
    if (hi - lo).abs() < f64::EPSILON {
        let pad = lo.abs() * 0.05 + 0.01;
        Some((lo - pad, hi + pad))
    } else {
        let pad = (hi - lo) * 0.1;
        Some((lo - pad, hi + pad))
    }
}

/// Format a chart axis label from **milliseconds** since Unix epoch; out-of-range → `"?"`.
///
/// Line-chart x values are in **seconds** — multiply by `1000.0` before calling.
fn format_time_axis(ts_ms: f64, intraday: bool) -> String {
    if !ts_ms.is_finite() {
        return "?".into();
    }
    let secs = ts_ms / 1000.0;
    if !(i64::MIN as f64..=i64::MAX as f64).contains(&secs) {
        return "?".into();
    }
    let Some(dt) = DateTime::<Utc>::from_timestamp(secs as i64, 0) else {
        return "?".into();
    };
    if intraday {
        dt.format("%m/%d %H:%MZ").to_string()
    } else {
        dt.format("%m/%d").to_string()
    }
}

fn charts_short_title(app: &App) -> String {
    let base = format!(
        "{} · {} · {}",
        app.symbol,
        app.time_range.label(),
        app.chart_mode.label()
    );
    if app.config.provider == MarketProviderKind::Polygon
        && app.charts_polygon_truncated
        && !app.charts_polygon_notice.is_empty()
    {
        format!("{base} · {}", app.charts_polygon_notice)
    } else {
        base
    }
}

fn charts_key_hints() -> &'static str {
    "1-4 range │ +/- zoom │ h l pan │ 0 reset │ c mode │ S SMA │ E EMA │ R RSI │ M MACD"
}

/// Start index of the visible window into `historical_data.results`.
fn viewport_global_start(vp: &ChartViewport, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let end = if vp.end == 0 {
        len
    } else {
        vp.end.min(len).max(1)
    };
    vp.start.min(end.saturating_sub(1))
}

/// Map visible OHLC bars to chart `(time_sec, value)` pairs for an aligned indicator series.
fn indicator_xy_visible(
    slice: &[HistoricalData],
    series: &[Option<f64>],
    global_start: usize,
) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for (j, bar) in slice.iter().enumerate() {
        let idx = global_start + j;
        if let Some(v) = series.get(idx).and_then(|x| *x) {
            if v.is_finite() {
                out.push((bar.t as f64 / 1000.0, v));
            }
        }
    }
    out
}

fn extend_price_bounds(base: (f64, f64), overlay: &[(f64, f64)]) -> (f64, f64) {
    let (mut lo, mut hi) = base;
    for &(_, y) in overlay {
        if y.is_finite() {
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    if (hi - lo).abs() < f64::EPSILON {
        let pad = lo.abs() * 0.05 + 0.01;
        (lo - pad, hi + pad)
    } else {
        let pad = (hi - lo) * 0.1;
        (lo - pad, hi + pad)
    }
}

/// Muted placeholder when RSI/MACD are enabled before historical data is ready.
fn draw_indicator_subpane_placeholder(f: &mut Frame, area: Rect, theme: ResolvedTheme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let line = Line::from(vec![Span::styled(
        "Waiting for chart data…",
        theme.fg_muted(),
    )]);
    f.render_widget(Paragraph::new(line).style(theme.canvas()), area);
}

fn oscillator_bounds(values: &[(f64, f64)], default_lo: f64, default_hi: f64) -> [f64; 2] {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for &(_, y) in values {
        if y.is_finite() {
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return [default_lo, default_hi];
    }
    if (hi - lo).abs() < f64::EPSILON {
        let pad = 0.5;
        [lo - pad, hi + pad]
    } else {
        let pad = (hi - lo) * 0.1;
        [lo - pad, hi + pad]
    }
}

fn charts_block_title(app: &App, include_key_hints: bool) -> String {
    if include_key_hints {
        format!("{} │ {}", charts_short_title(app), charts_key_hints())
    } else {
        charts_short_title(app)
    }
}

pub fn draw_charts(
    f: &mut Frame,
    app: &App,
    area: Rect,
    theme: ResolvedTheme,
    layout: ResolvedLayout,
) {
    if layout.charts_chart_pct >= 100 {
        draw_charts_inner(f, app, area, theme, true);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(layout.charts_chart_pct),
            Constraint::Min(2),
        ])
        .split(area);
    draw_charts_inner(f, app, chunks[0], theme, false);
    draw_charts_chrome_strip(f, app, chunks[1], theme);
}

fn draw_charts_chrome_strip(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme) {
    let line = Line::from(vec![Span::styled(
        format!("{} │ {}", charts_short_title(app), charts_key_hints()),
        theme.fg_muted(),
    )]);
    let block = Block::default()
        .borders(Borders::TOP)
        .style(theme.canvas())
        .border_style(Style::default().fg(theme.border).bg(theme.background));
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_charts_inner(f: &mut Frame, app: &App, area: Rect, theme: ResolvedTheme, full_title: bool) {
    let block = Block::default()
        .title(charts_block_title(app, full_title))
        .borders(Borders::ALL)
        .style(theme.canvas())
        .border_style(Style::default().fg(theme.border).bg(theme.background));

    let Some(historical_data) = &app.historical_data else {
        let loading_text = Line::from(vec![Span::styled(
            "Loading historical data...",
            theme.warning_text(),
        )]);
        let paragraph = ratatui::widgets::Paragraph::new(loading_text).block(block);
        f.render_widget(paragraph, area);
        return;
    };

    if historical_data.results.is_empty() {
        let no_data_text = Line::from(vec![Span::styled(
            "No historical data available",
            theme.error_text(),
        )]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let slice = visible_slice(&historical_data.results, &app.chart_viewport);
    if slice.is_empty() {
        let no_data_text = Line::from(vec![Span::styled(
            "No data in current view",
            theme.error_text(),
        )]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let Some((price_min, price_max)) = price_bounds(slice) else {
        let no_data_text = Line::from(vec![Span::styled(
            "Invalid price data",
            theme.error_text(),
        )]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
        return;
    };

    let data: Vec<(f64, f64)> = slice
        .iter()
        .map(|b| (b.t as f64 / 1000.0, b.c))
        .collect();

    let (min_time, max_time) = data.iter().fold((f64::MAX, f64::MIN), |(a, b), &(t, _)| {
        (a.min(t), b.max(t))
    });
    let span_sec = max_time - min_time;
    let intraday = matches!(app.time_range, TimeRange::D1 | TimeRange::W1) || span_sec < 86400.0 * 3.0;

    let first_ts = slice.first().map(|b| b.t as f64).unwrap_or(0.0);
    let last_ts = slice.last().map(|b| b.t as f64).unwrap_or(0.0);
    let vis_from = format_time_axis(first_ts, intraday);
    let vis_to = format_time_axis(last_ts, intraday);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let global_start = viewport_global_start(&app.chart_viewport, historical_data.results.len());
    let cache = app.chart_indicator_cache.as_ref();

    let render_price = |f: &mut Frame, price_area: Rect| {
        if matches!(app.chart_mode, ChartDisplayMode::Candlestick) {
            if slice.len() < 2 {
                let msg = Line::from(vec![Span::styled(
                    format!(
                        "Candles need 2+ bars (visible {}–{}, {} bar(s)). Press `c` for line.",
                        vis_from, vis_to, slice.len()
                    ),
                    theme.warning_text(),
                )]);
                f.render_widget(Paragraph::new(msg).style(theme.canvas()), price_area);
                return;
            }
            let chart = CandlestickChart {
                data: slice,
                min_y: price_min,
                max_y: price_max,
                theme,
            };
            f.render_widget(chart, price_area);
            if app.chart_indicators.overlay_enabled() {
                let hint = Line::from(vec![Span::styled(
                    "Indicators: press `c` for line chart (SMA/EMA overlays)",
                    theme.fg_muted(),
                )]);
                let hint_area = Rect {
                    y: price_area
                        .y
                        .saturating_add(price_area.height.saturating_sub(1)),
                    height: 1,
                    ..price_area
                };
                if hint_area.height > 0 && hint_area.width > 0 {
                    f.render_widget(Paragraph::new(hint).style(theme.canvas()), hint_area);
                }
            }
            return;
        }

        let mut sma_data: Vec<(f64, f64)> = Vec::new();
        let mut ema_data: Vec<(f64, f64)> = Vec::new();
        if let Some(c) = cache {
            if app.chart_indicators.sma_20 {
                sma_data = indicator_xy_visible(slice, &c.sma_20, global_start);
            }
            if app.chart_indicators.ema_20 {
                ema_data = indicator_xy_visible(slice, &c.ema_20, global_start);
            }
        }

        let mut y_bounds = (price_min, price_max);
        y_bounds = extend_price_bounds(y_bounds, &sma_data);
        y_bounds = extend_price_bounds(y_bounds, &ema_data);
        let (y_min, y_max) = y_bounds;

        let mut datasets = vec![Dataset::default()
            .name("Close")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.fg_accent())
            .data(&data)];
        if !sma_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("SMA(20)")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(theme.fg_muted())
                    .data(&sma_data),
            );
        }
        if !ema_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("EMA(20)")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(theme.fg_positive())
                    .data(&ema_data),
            );
        }

        let format_time = |time: &f64| format_time_axis(*time * 1000.0, intraday);
        let format_price = |price: &f64| crate::app::format::format_usd_price(*price);

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title(Line::from(vec![Span::styled(
                        format!("UTC  {vis_from} → {vis_to}"),
                        theme.fg_foreground(),
                    )]))
                    .style(theme.fg_foreground())
                    .bounds([min_time, max_time])
                    .labels(vec![
                        Span::styled(format_time(&min_time), theme.fg_foreground()),
                        Span::styled(
                            format_time(&((min_time + max_time) / 2.0)),
                            theme.fg_foreground(),
                        ),
                        Span::styled(format_time(&max_time), theme.fg_foreground()),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title(Line::from(vec![Span::styled(
                        "Price",
                        theme.fg_foreground(),
                    )]))
                    .style(theme.fg_foreground())
                    .bounds([y_min, y_max])
                    .labels(vec![
                        Span::styled(format_price(&y_min), theme.fg_foreground()),
                        Span::styled(
                            format_price(&((y_min + y_max) / 2.0)),
                            theme.fg_foreground(),
                        ),
                        Span::styled(format_price(&y_max), theme.fg_foreground()),
                    ]),
            );
        f.render_widget(chart, price_area);
    };

    let render_rsi = |f: &mut Frame, rsi_area: Rect| {
        let Some(c) = cache else {
            return;
        };
        let rsi_data = indicator_xy_visible(slice, &c.rsi_14, global_start);
        if rsi_data.is_empty() {
            return;
        }
        let ref_30 = vec![(min_time, 30.0), (max_time, 30.0)];
        let ref_70 = vec![(min_time, 70.0), (max_time, 70.0)];
        let datasets = vec![
            Dataset::default()
                .name("RSI(14)")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(theme.fg_accent())
                .data(&rsi_data),
            Dataset::default()
                .name("30")
                .graph_type(GraphType::Line)
                .style(theme.fg_muted())
                .data(&ref_30),
            Dataset::default()
                .name("70")
                .graph_type(GraphType::Line)
                .style(theme.fg_muted())
                .data(&ref_70),
        ];
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("RSI(14)")
                    .borders(Borders::TOP)
                    .style(theme.canvas())
                    .border_style(Style::default().fg(theme.border).bg(theme.background)),
            )
            .x_axis(
                Axis::default()
                    .style(theme.fg_foreground())
                    .bounds([min_time, max_time]),
            )
            .y_axis(
                Axis::default()
                    .style(theme.fg_foreground())
                    .bounds([0.0, 100.0]),
            );
        f.render_widget(chart, rsi_area);
    };

    let render_macd = |f: &mut Frame, macd_area: Rect| {
        let Some(c) = cache else {
            return;
        };
        let macd_data = indicator_xy_visible(slice, &c.macd.macd, global_start);
        let signal_data = indicator_xy_visible(slice, &c.macd.signal, global_start);
        let hist_data = indicator_xy_visible(slice, &c.macd.histogram, global_start);
        if macd_data.is_empty() && signal_data.is_empty() && hist_data.is_empty() {
            return;
        }
        let mut all = Vec::new();
        all.extend_from_slice(&macd_data);
        all.extend_from_slice(&signal_data);
        all.extend_from_slice(&hist_data);
        let y_bounds = oscillator_bounds(&all, -1.0, 1.0);

        let mut datasets = Vec::new();
        if !hist_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Hist")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(theme.fg_border())
                    .data(&hist_data),
            );
        }
        if !macd_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("MACD")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(theme.fg_accent())
                    .data(&macd_data),
            );
        }
        if !signal_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Signal")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(theme.fg_positive())
                    .data(&signal_data),
            );
        }
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("MACD 12/26/9")
                    .borders(Borders::TOP)
                    .style(theme.canvas())
                    .border_style(Style::default().fg(theme.border).bg(theme.background)),
            )
            .x_axis(
                Axis::default()
                    .style(theme.fg_foreground())
                    .bounds([min_time, max_time]),
            )
            .y_axis(
                Axis::default()
                    .style(theme.fg_foreground())
                    .bounds(y_bounds),
            );
        f.render_widget(chart, macd_area);
    };

    if app.chart_indicators.needs_subpane() {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Min(4)])
            .split(inner);
        render_price(f, main_chunks[0]);

        let sub_constraints = match (app.chart_indicators.rsi_14, app.chart_indicators.macd) {
            (true, true) => vec![Constraint::Ratio(1, 1), Constraint::Ratio(1, 1)],
            _ => vec![Constraint::Min(3)],
        };
        let sub_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(sub_constraints)
            .split(main_chunks[1]);
        let mut idx = 0;
        if app.chart_indicators.rsi_14 {
            if cache.is_some() {
                render_rsi(f, sub_chunks[idx]);
            } else {
                draw_indicator_subpane_placeholder(f, sub_chunks[idx], theme);
            }
            idx += 1;
        }
        if app.chart_indicators.macd {
            if cache.is_some() {
                render_macd(f, sub_chunks[idx]);
            } else {
                draw_indicator_subpane_placeholder(f, sub_chunks[idx], theme);
            }
        }
    } else {
        render_price(f, inner);
    }
}

/// Candlesticks in equal-width slots so bars sit closer than edge-to-edge indexing (Issue #7).
struct CandlestickChart<'a> {
    data: &'a [HistoricalData],
    min_y: f64,
    max_y: f64,
    theme: ResolvedTheme,
}

impl CandlestickChart<'_> {
    /// Center of bar `i` in slot `i` of `n` equal columns (tighter than edge-to-edge `i/(n-1)`).
    fn slot_center_x(&self, area: Rect, i: usize, n: usize) -> u16 {
        let w = area.width.max(1);
        if n == 0 {
            return area.left();
        }
        if n == 1 {
            return area.left() + w / 2;
        }
        let slot = f64::from(w) / n as f64;
        let cx = f64::from(area.left()) + slot * (i as f64 + 0.5);
        cx.round().clamp(f64::from(area.left()), f64::from(area.right().saturating_sub(1))) as u16
    }

    fn body_width_cells(&self, area: Rect, n: usize) -> u16 {
        if n == 0 {
            return 1;
        }
        let slot = f64::from(area.width.max(1)) / n as f64;
        if slot >= 4.0 {
            2
        } else {
            1
        }
    }

    fn price_to_row(&self, area: Rect, price: f64) -> Option<u16> {
        let h = area.height;
        if h < 2 {
            return None;
        }
        let span = self.max_y - self.min_y;
        if span.abs() < f64::EPSILON {
            return Some(area.top() + h / 2);
        }
        let frac = (price - self.min_y) / span;
        let row = area.bottom().saturating_sub(1) as f64 - frac * f64::from(h.saturating_sub(1));
        let r = row.round() as i32;
        let top = i32::from(area.top());
        let bottom = i32::from(area.bottom().saturating_sub(1));
        Some(r.clamp(top, bottom) as u16)
    }
}

impl Widget for CandlestickChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.data.is_empty() || area.width < 2 || area.height < 2 {
            return;
        }
        let bg = self.theme.background;
        let fg = self.theme.foreground;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(" ");
                cell.set_fg(fg);
                cell.set_bg(bg);
            }
        }
        let n = self.data.len();

        for (i, bar) in self.data.iter().enumerate() {
            let cx = self.slot_center_x(area, i, n);
            let bw = self.body_width_cells(area, n);
            let x0 = cx.saturating_sub(bw / 2);
            let Some(y_high) = self.price_to_row(area, bar.h) else {
                continue;
            };
            let Some(y_low) = self.price_to_row(area, bar.l) else {
                continue;
            };
            let Some(y_open) = self.price_to_row(area, bar.o) else {
                continue;
            };
            let Some(y_close) = self.price_to_row(area, bar.c) else {
                continue;
            };

            let up = bar.c >= bar.o;
            let color = if up {
                self.theme.positive
            } else {
                self.theme.negative
            };

            let y_wick_top = y_high.min(y_low);
            let y_wick_bot = y_high.max(y_low);
            for y in y_wick_top..=y_wick_bot {
                let cell = buf.get_mut(cx, y);
                cell.set_symbol(symbols::line::VERTICAL);
                cell.set_fg(color);
                cell.set_bg(bg);
            }

            let body_top = y_open.min(y_close);
            let mut body_bot = y_open.max(y_close);
            if body_top == body_bot {
                body_bot = (body_bot + 1).min(area.bottom().saturating_sub(1));
            }
            for y in body_top..=body_bot {
                for dx in 0..bw {
                    let xx = x0.saturating_add(dx).min(area.right().saturating_sub(1));
                    let cell = buf.get_mut(xx, y);
                    cell.set_symbol("█");
                    cell.set_fg(color);
                    cell.set_bg(bg);
                }
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(t: u64, o: f64, h: f64, l: f64, c: f64) -> HistoricalData {
        HistoricalData {
            o,
            h,
            l,
            c,
            v: 1.0,
            t,
            vw: c,
            n: None,
        }
    }

    #[test]
    fn visible_slice_full_when_end_zero() {
        let v = vec![bar(1, 1.0, 2.0, 0.5, 1.5), bar(2, 1.5, 2.5, 1.0, 2.0)];
        let vp = ChartViewport::default();
        let s = visible_slice(&v, &vp);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn visible_slice_window() {
        let v: Vec<_> = (0..10)
            .map(|i| bar(i * 1000, 1.0, 2.0, 0.5, 1.0))
            .collect();
        let vp = ChartViewport { start: 2, end: 6 };
        let s = visible_slice(&v, &vp);
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn zoom_in_shrinks_width() {
        let mut vp = ChartViewport::full(20);
        viewport_zoom_in(&mut vp, 20);
        assert!(vp.width() < 20);
        assert!(vp.width() >= 2);
    }

    #[test]
    fn pan_left_moves_window() {
        let mut vp = ChartViewport { start: 5, end: 15 };
        viewport_pan_left(&mut vp, 20);
        assert_eq!(vp.start, 4);
        assert_eq!(vp.end, 14);
    }

    #[test]
    fn pan_right_at_end_noop() {
        let mut vp = ChartViewport { start: 10, end: 20 };
        viewport_pan_right(&mut vp, 20);
        assert_eq!(vp.start, 10);
        assert_eq!(vp.end, 20);
    }

    fn hist(ticker: &str, bars: usize) -> HistoricalResponse {
        HistoricalResponse {
            ticker: ticker.to_string(),
            results: (0..bars)
                .map(|i| bar(i as u64 * 1000, 1.0, 2.0, 0.5, 1.0))
                .collect(),
            status: "OK".into(),
            request_id: String::new(),
            count: bars as u32,
            ..Default::default()
        }
    }

    #[test]
    fn refresh_no_previous_is_full() {
        let new = hist("AAPL", 5);
        let vp = chart_viewport_after_refresh(None, ChartViewport::default(), &new, "AAPL");
        assert_eq!(vp, ChartViewport::full(5));
    }

    #[test]
    fn refresh_ticker_change_resets_full() {
        let prev = hist("AAPL", 10);
        let new = hist("MSFT", 10);
        let vp = chart_viewport_after_refresh(
            Some(&prev),
            ChartViewport {
                start: 2,
                end: 8,
            },
            &new,
            "MSFT",
        );
        assert_eq!(vp, ChartViewport::full(10));
    }

    #[test]
    fn refresh_preserves_zoomed_viewport() {
        let prev = hist("AAPL", 20);
        let new = hist("AAPL", 20);
        let vp = chart_viewport_after_refresh(
            Some(&prev),
            ChartViewport { start: 5, end: 15 },
            &new,
            "AAPL",
        );
        assert_eq!(vp.start, 5);
        assert_eq!(vp.end, 15);
    }

    #[test]
    fn refresh_empty_response_ticker_matches_requested() {
        let prev = hist("AAPL", 20);
        let new = hist("", 20);
        let vp = chart_viewport_after_refresh(
            Some(&prev),
            ChartViewport { start: 5, end: 15 },
            &new,
            "AAPL",
        );
        assert_eq!(vp.start, 5);
        assert_eq!(vp.end, 15);
    }

    #[test]
    fn refresh_full_series_grows_with_new_bars() {
        let prev = hist("AAPL", 30);
        let new = hist("AAPL", 31);
        let vp = chart_viewport_after_refresh(Some(&prev), ChartViewport::full(30), &new, "AAPL");
        assert_eq!(vp, ChartViewport::full(31));
    }

    #[test]
    fn clamp_viewport_when_series_shortens() {
        let vp = ChartViewport { start: 8, end: 20 };
        let out = clamp_viewport_to_len(vp, 15);
        assert_eq!(out.end, 15);
        assert!(out.start < out.end);
    }

    /// Issue #36 / §40.1 — invalid timestamps must not panic; axis shows `?`.
    #[test]
    fn format_time_axis_valid_recent_ms() {
        let label = format_time_axis(1_700_000_000_000.0, false);
        assert_ne!(label, "?");
        assert!(label.contains('/'));
    }

    #[test]
    fn format_time_axis_invalid_returns_question_mark() {
        for ts in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 9e18] {
            assert_eq!(format_time_axis(ts, false), "?");
            assert_eq!(format_time_axis(ts, true), "?");
        }
    }

    #[test]
    fn format_time_axis_epoch_is_stable() {
        let label = format_time_axis(0.0, false);
        assert!(!label.is_empty());
    }
}
