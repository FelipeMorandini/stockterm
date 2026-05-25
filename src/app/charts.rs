//! Charts tab: line chart, candlesticks, viewport, indicators (Issues #7 / #8 / #9, #21).

use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::config::{MarketProviderKind, ResolvedLayout};
use crate::indicators::types::IndicatorSeries;
use crate::indicators::{
    ema, macd, rsi, sma, MacdOutput, EMA_PERIOD, MACD_FAST, MACD_SIGNAL, MACD_SLOW, RSI_PERIOD,
    SMA_PERIOD,
};
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

    /// Stable id in `~/.stockterm.json` (`last_chart_mode`) — Issue #180 / SPEC §54.
    pub const fn as_config_str(self) -> &'static str {
        match self {
            ChartDisplayMode::Line => "line",
            ChartDisplayMode::Candlestick => "candles",
        }
    }

    /// Parse persisted chart display mode; unknown strings return `None`.
    pub fn from_config_str(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "line" | "Line" => ChartDisplayMode::Line,
            "candles" | "Candles" | "candlestick" | "Candlestick" => ChartDisplayMode::Candlestick,
            _ => return None,
        })
    }

    fn label(self) -> &'static str {
        self.as_config_str()
    }
}

/// Precomputed candlestick body layout (Issue #199 / §64).
#[derive(Debug, Clone)]
pub struct ChartCandleLayoutCache {
    pub(crate) key: ChartCandleLayoutKey,
    pub(crate) layouts: Vec<CandleBarLayout>,
}

/// Cache key for candlestick layout invalidation (Issue #199 / §64.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChartCandleLayoutKey {
    pub(crate) area: Rect,
    pub(crate) viewport: ChartViewport,
    pub(crate) bars_len: usize,
    pub(crate) time_range: TimeRange,
    pub(crate) series_stamp: u64,
}

/// Pure helper: computes the price pane Rect for candlestick drawing (Issue #199 / §64.2).
///
/// Matches `draw_charts_inner`: bordered block **with** title, then optional RSI/MACD split.
pub fn charts_price_area(area: Rect, block_title: &str, indicators: ChartIndicatorToggles) -> Rect {
    let inner = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .inner(area);
    if !indicators.needs_subpane() {
        return inner;
    }
    let panes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Min(4)])
        .split(inner);
    panes[0]
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
pub fn visible_slice<'a>(
    results: &'a [HistoricalData],
    vp: &ChartViewport,
) -> &'a [HistoricalData] {
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

fn effective_series_ticker<'a>(
    series: &'a HistoricalResponse,
    requested_symbol: &'a str,
) -> &'a str {
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
    app: &mut App,
    area: Rect,
    theme: ResolvedTheme,
    layout: ResolvedLayout,
) {
    let full_title = layout.charts_chart_pct >= 100;
    let (chart_area, chrome_area) = if full_title {
        (area, None)
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(layout.charts_chart_pct),
                Constraint::Min(2),
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    };

    if matches!(app.chart_mode, ChartDisplayMode::Candlestick) {
        let block_title = charts_block_title(app, full_title);
        let price_area = charts_price_area(chart_area, &block_title, app.chart_indicators);
        app.prepare_charts_draw_cache(price_area);
    }

    match chrome_area {
        None => draw_charts_inner(f, app, chart_area, theme, true),
        Some(chrome) => {
            draw_charts_inner(f, app, chart_area, theme, false);
            draw_charts_chrome_strip(f, app, chrome, theme);
        }
    }
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
    let block_title = charts_block_title(app, full_title);
    let block = Block::default()
        .title(block_title.as_str())
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
        let no_data_text = Line::from(vec![Span::styled("Invalid price data", theme.error_text())]);
        let paragraph = ratatui::widgets::Paragraph::new(no_data_text).block(block);
        f.render_widget(paragraph, area);
        return;
    };

    let data: Vec<(f64, f64)> = slice.iter().map(|b| (b.t as f64 / 1000.0, b.c)).collect();

    let (min_time, max_time) = data
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &(t, _)| (a.min(t), b.max(t)));
    let span_sec = max_time - min_time;
    let intraday =
        matches!(app.time_range, TimeRange::D1 | TimeRange::W1) || span_sec < 86400.0 * 3.0;

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
                        vis_from,
                        vis_to,
                        slice.len()
                    ),
                    theme.warning_text(),
                )]);
                f.render_widget(Paragraph::new(msg).style(theme.canvas()), price_area);
                return;
            }
            let layouts = app
                .chart_candle_layout
                .as_ref()
                .map(|c| c.layouts.as_slice())
                .unwrap_or(&[]);
            let chart = CandlestickChart {
                data: slice,
                layouts,
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

    let price_area = charts_price_area(area, &block_title, app.chart_indicators);

    if app.chart_indicators.needs_subpane() {
        render_price(f, price_area);

        let sub_area = Rect {
            x: inner.x,
            y: price_area.bottom(),
            width: inner.width,
            height: inner.bottom().saturating_sub(price_area.bottom()),
        };
        let sub_constraints = match (app.chart_indicators.rsi_14, app.chart_indicators.macd) {
            (true, true) => vec![Constraint::Ratio(1, 1), Constraint::Ratio(1, 1)],
            _ => vec![Constraint::Min(3)],
        };
        let sub_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(sub_constraints)
            .split(sub_area);
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
        render_price(f, price_area);
    }
}

/// Fixed-width candle body span per bar (Issue #190 — layout rewrite).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CandleBarLayout {
    body_left: u16,
    body_right: u16,
}

impl CandleBarLayout {
    /// Body width in terminal columns (inclusive span).
    fn body_w(self) -> u16 {
        self.body_right.saturating_sub(self.body_left) + 1
    }

    /// Wick column — geometric center of the body (`body_left + body_w / 2`, correct for even widths).
    fn wick_x(self) -> u16 {
        self.body_left + self.body_w() / 2
    }
}

/// Max body width in terminal cells; gap between bodies when they fit.
const CANDLE_BODY_W_MAX: u16 = 3;
const CANDLE_GAP_MIN: u16 = 1;

/// Map bar timestamp to a column in `[left, right]` (line-chart time scale).
fn time_to_column(t: u64, t0: u64, t1: u64, left: u16, right: u16) -> u16 {
    if t1 <= t0 {
        return (left + right) / 2;
    }
    let span = (t1 - t0) as f64;
    let frac = (t.saturating_sub(t0) as f64) / span;
    let x = f64::from(left) + frac * f64::from(right.saturating_sub(left));
    x.round().clamp(f64::from(left), f64::from(right)) as u16
}

/// Median seconds between consecutive bar timestamps (§65: `t` is always ms).
fn median_bar_gap_secs(bars: &[HistoricalData]) -> u64 {
    if bars.len() < 2 {
        return 86_400;
    }
    let mut gaps: Vec<u64> = bars
        .windows(2)
        .map(|w| w[1].t.saturating_sub(w[0].t))
        .collect();
    gaps.sort_unstable();
    gaps[gaps.len() / 2] / 1_000
}

/// Minimum median bar gap before **Y1** uses clock-time x (weekly+ bars only).
const TIME_LAYOUT_MIN_GAP_SECS: u64 = 7 * 86_400;

/// Pick `(body_w, gap)` for **index** layout (**1D** / **1W** / dense daily **M1**).
///
/// Always uses at least [`CANDLE_GAP_MIN`] between bodies when not dense.
fn fit_candle_body_and_gap_index(width: u16, n: usize) -> (u16, u16, bool) {
    let n = n as u16;
    if n == 0 {
        return (1, 0, true);
    }
    for body_w in (1..=CANDLE_BODY_W_MAX).rev() {
        let gap = CANDLE_GAP_MIN;
        let stride = body_w + gap;
        let total = n.saturating_mul(stride).saturating_sub(gap);
        if total <= width {
            return (body_w, gap, false);
        }
    }
    (1, 0, true)
}

/// Pick `(body_w, gap)` for **time** layout (**Y1** weekly+ series).
fn fit_candle_body_and_gap_time(width: u16, n: usize) -> (u16, u16, bool) {
    let n = n as u16;
    if n == 0 {
        return (1, 0, true);
    }
    for body_w in (1..=CANDLE_BODY_W_MAX).rev() {
        for gap in (0..=CANDLE_GAP_MIN).rev() {
            let stride = body_w + gap;
            let total = n.saturating_mul(stride).saturating_sub(gap);
            if total <= width {
                return (body_w, gap, false);
            }
        }
    }
    (1, 0, true)
}

fn layout_from_centers(
    centers: &[u16],
    body_w: u16,
    left: u16,
    right: u16,
) -> Vec<CandleBarLayout> {
    let half = (body_w.saturating_sub(1)) / 2;
    centers
        .iter()
        .map(|&cx| {
            let bl = cx.saturating_sub(half).max(left);
            let br = (bl + body_w.saturating_sub(1)).min(right);
            CandleBarLayout {
                body_left: bl,
                body_right: br,
            }
        })
        .collect()
}

/// One column per bar when `n > width` (intraday full-range view).
fn layout_candles_dense(area: Rect, n: usize) -> Vec<CandleBarLayout> {
    let w = area.width.max(1);
    let left = area.left();
    let right = area.right().saturating_sub(1);
    (0..n)
        .map(|i| {
            let x0 = left.saturating_add(((i as u32 * u32::from(w)) / n as u32) as u16);
            let x1_ex = left.saturating_add((((i + 1) as u32 * u32::from(w)) / n as u32) as u16);
            let x1 = if x1_ex > x0 {
                x1_ex.saturating_sub(1).min(right)
            } else {
                x0.min(right)
            };
            CandleBarLayout {
                body_left: x0,
                body_right: x1,
            }
        })
        .collect()
}

/// Evenly spaced fixed-width bodies with mandatory gap ( **1D** / **1W** / daily **M1** ).
fn layout_candles_index(area: Rect, n: usize, body_w: u16, gap: u16) -> Vec<CandleBarLayout> {
    let left = area.left();
    let right = area.right().saturating_sub(1);
    let w = area.width.max(1);
    let stride = body_w + gap;
    let total = (n as u16).saturating_mul(stride).saturating_sub(gap);
    let start = left + (w.saturating_sub(total)) / 2;
    (0..n)
        .map(|i| {
            let bl = start.saturating_add(stride.saturating_mul(i as u16));
            CandleBarLayout {
                body_left: bl,
                body_right: (bl + body_w.saturating_sub(1)).min(right),
            }
        })
        .collect()
}

/// Time-scaled centers with fixed body width; only resolve overlaps (preserves weekend gaps).
fn layout_candles_time(area: Rect, bars: &[HistoricalData], body_w: u16) -> Vec<CandleBarLayout> {
    let n = bars.len();
    let left = area.left();
    let right = area.right().saturating_sub(1);
    let t0 = bars.first().map(|b| b.t).unwrap_or(0);
    let t1 = bars.last().map(|b| b.t).unwrap_or(t0);
    let mut centers: Vec<u16> = bars
        .iter()
        .map(|b| time_to_column(b.t, t0, t1, left, right))
        .collect();

    for i in 1..n {
        let min_cx = centers[i - 1].saturating_add(body_w);
        if centers[i] < min_cx {
            centers[i] = min_cx;
        }
    }
    let half = (body_w.saturating_sub(1)) / 2;
    let max_cx = right.saturating_sub(half);
    if let Some(&last) = centers.last() {
        if last > max_cx {
            let shift = last - max_cx;
            for c in &mut centers {
                *c = c.saturating_sub(shift).max(left.saturating_add(half));
            }
        }
    }

    layout_from_centers(&centers, body_w, left, right)
}

/// Compute per-bar layout for the visible OHLC slice (Issue #190 / §63).
pub(crate) fn layout_candles(
    area: Rect,
    bars: &[HistoricalData],
    time_range: TimeRange,
) -> Vec<CandleBarLayout> {
    let n = bars.len();
    if n == 0 {
        return Vec::new();
    }
    let left = area.left();
    let right = area.right().saturating_sub(1);
    let w = area.width.max(1);

    if n == 1 {
        let body_w = w.clamp(1, CANDLE_BODY_W_MAX);
        let x0 = left + (w.saturating_sub(body_w)) / 2;
        return vec![CandleBarLayout {
            body_left: x0,
            body_right: (x0 + body_w.saturating_sub(1)).min(right),
        }];
    }

    // **Y1** weekly+ bars: clock-time x. **M1** daily bars: even index (see `median_bar_gap_secs`).
    let use_time = matches!(time_range, TimeRange::Y1)
        && median_bar_gap_secs(bars) >= TIME_LAYOUT_MIN_GAP_SECS;

    let (body_w, gap, dense) = if use_time {
        fit_candle_body_and_gap_time(w, n)
    } else {
        fit_candle_body_and_gap_index(w, n)
    };
    if dense {
        return layout_candles_dense(area, n);
    }

    if use_time {
        layout_candles_time(area, bars, body_w)
    } else {
        layout_candles_index(area, n, body_w, gap)
    }
}

/// Candlesticks in integer-width slots for column-stable density (Issues #7, #190, #199).
struct CandlestickChart<'a> {
    data: &'a [HistoricalData],
    layouts: &'a [CandleBarLayout],
    min_y: f64,
    max_y: f64,
    theme: ResolvedTheme,
}

impl CandlestickChart<'_> {
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
        debug_assert_eq!(
            self.layouts.len(),
            self.data.len(),
            "layouts must match data length (precomputed in update phase)"
        );

        for (bar, layout) in self.data.iter().zip(self.layouts.iter()) {
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

            let body_top = y_open.min(y_close);
            let mut body_bot = y_open.max(y_close);
            if body_top == body_bot {
                body_bot = (body_bot + 1).min(area.bottom().saturating_sub(1));
            }

            let wick_x = layout.wick_x();
            for y in y_wick_top..=y_wick_bot {
                let cell = buf.get_mut(wick_x, y);
                cell.set_symbol(symbols::line::VERTICAL);
                cell.set_fg(color);
                cell.set_bg(bg);
            }

            for y in body_top..=body_bot {
                for x in layout.body_left..=layout.body_right {
                    let cell = buf.get_mut(x, y);
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
    fn chart_display_mode_from_config_str() {
        assert_eq!(
            ChartDisplayMode::from_config_str("line"),
            Some(ChartDisplayMode::Line)
        );
        assert_eq!(
            ChartDisplayMode::from_config_str("candles"),
            Some(ChartDisplayMode::Candlestick)
        );
        assert_eq!(
            ChartDisplayMode::from_config_str("candlestick"),
            Some(ChartDisplayMode::Candlestick)
        );
        assert!(ChartDisplayMode::from_config_str("invalid").is_none());
        assert_eq!(ChartDisplayMode::Line.as_config_str(), "line");
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
        let v: Vec<_> = (0..10).map(|i| bar(i * 1000, 1.0, 2.0, 0.5, 1.0)).collect();
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
            ChartViewport { start: 2, end: 8 },
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

    fn chart_area(width: u16) -> Rect {
        Rect::new(0, 0, width, 20)
    }

    const DAY_SEC: u64 = 86_400;
    const TS_BASE: u64 = 1_700_000_000_000;

    fn daily_bars(count: usize, day_stride: u64) -> Vec<HistoricalData> {
        (0..count)
            .map(|i| {
                let t = TS_BASE + i as u64 * day_stride * DAY_SEC * 1000;
                let base = 100.0 + f64::from(i as u32);
                bar(t, base, base + 2.0, base - 0.5, base + 1.0)
            })
            .collect()
    }

    fn daily_bars_ms(count: usize) -> Vec<HistoricalData> {
        daily_bars(count, 1)
    }

    fn intraday_bars(count: usize) -> Vec<HistoricalData> {
        const FIVE_MIN_MS: u64 = 300_000;
        (0..count)
            .map(|i| {
                let t = TS_BASE + i as u64 * FIVE_MIN_MS;
                let base = 100.0 + f64::from(i as u32) * 0.01;
                bar(t, base, base + 0.5, base - 0.2, base + 0.1)
            })
            .collect()
    }

    #[test]
    fn layout_candles_partition_80_20() {
        let area = chart_area(60);
        let bars = daily_bars(20, 1);
        let layouts = layout_candles(area, &bars, TimeRange::M1);
        assert_eq!(layouts.len(), 20);
        assert_index_candle_layout_invariants(&layouts, area);
    }

    #[test]
    fn layout_candles_single_bar() {
        let area = chart_area(60);
        let bars = [bar(TS_BASE, 100.0, 102.0, 99.0, 101.0)];
        let layouts = layout_candles(area, &bars, TimeRange::M1);
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].body_left, area.left() + 28);
        assert_eq!(layouts[0].body_right, layouts[0].body_left + 2);
    }

    #[test]
    fn layout_candles_fit_body_and_gap() {
        let (body_w, gap, dense) = fit_candle_body_and_gap_index(60, 20);
        assert_eq!((body_w, gap, dense), (2, 1, false));
        let (body_w, gap, dense) = fit_candle_body_and_gap_index(80, 40);
        assert_eq!((body_w, gap, dense), (1, 1, false));
        let (body_w, gap, dense) = fit_candle_body_and_gap_index(10, 25);
        assert_eq!((body_w, gap, dense), (1, 0, true));
    }

    #[test]
    fn layout_candles_wick_centered_on_even_width_body() {
        let layout = CandleBarLayout {
            body_left: 10,
            body_right: 11,
        };
        assert_eq!(layout.wick_x(), 11);
        assert!(layout.wick_x() >= layout.body_left);
        assert!(layout.wick_x() <= layout.body_right);
    }

    #[test]
    fn layout_candles_d1_bodies_do_not_touch() {
        let area = chart_area(80);
        let bars = intraday_bars(40);
        let layouts = layout_candles(area, &bars, TimeRange::D1);
        for pair in layouts.windows(2) {
            assert!(pair[1].body_left > pair[0].body_right);
        }
    }

    /// **Y1** weekly+: irregular timestamp gaps widen time-mapped x more than uniform steps.
    #[test]
    fn layout_candles_time_irregular_gap_spread() {
        let area = chart_area(80);
        let right = area.right().saturating_sub(1);
        let bars = vec![
            bar(TS_BASE, 100.0, 102.0, 99.0, 101.0),
            bar(TS_BASE + DAY_SEC * 7 * 1000, 101.0, 103.0, 100.0, 102.0),
            bar(TS_BASE + DAY_SEC * 14 * 1000, 102.0, 104.0, 101.0, 103.0),
            bar(TS_BASE + DAY_SEC * 28 * 1000, 103.0, 105.0, 102.0, 104.0),
        ];
        assert!(median_bar_gap_secs(&bars) >= TIME_LAYOUT_MIN_GAP_SECS);
        let t0 = bars[0].t;
        let t1 = bars[3].t;
        let centers: Vec<u16> = bars
            .iter()
            .map(|b| time_to_column(b.t, t0, t1, area.left(), right))
            .collect();
        let adjacent = centers[2] - centers[1];
        let wide = centers[3] - centers[2];
        assert!(wide > adjacent, "wide={wide} adjacent={adjacent}");
        let layouts = layout_candles(area, &bars, TimeRange::Y1);
        assert_eq!(layouts.len(), 4);
        assert_index_candle_layout_invariants(&layouts, area);
        assert!(layouts[3].body_left > layouts[2].body_right);
    }

    #[test]
    fn m1_daily_bars_use_index_not_time_layout() {
        let bars = daily_bars_ms(22);
        assert_eq!(median_bar_gap_secs(&bars), DAY_SEC);
        assert!(median_bar_gap_secs(&bars) < TIME_LAYOUT_MIN_GAP_SECS);
        let area = chart_area(80);
        let layouts = layout_candles(area, &bars, TimeRange::M1);
        let (body_w, gap, _) = fit_candle_body_and_gap_index(area.width, bars.len());
        let stride = body_w + gap;
        assert_eq!(layouts[1].body_left - layouts[0].body_left, stride);
        for pair in layouts.windows(2) {
            assert_eq!(pair[1].body_left, pair[0].body_right + gap + 1);
        }
    }

    #[test]
    fn charts_median_bar_gap_secs_only_path() {
        let bars = daily_bars_ms(5);
        assert_eq!(median_bar_gap_secs(&bars), DAY_SEC);
        let intraday = intraday_bars(5);
        assert_eq!(median_bar_gap_secs(&intraday), 300);
    }

    #[test]
    fn m1_intraday_fallback_uses_index_stride() {
        let bars = intraday_bars(30);
        assert!(median_bar_gap_secs(&bars) < TIME_LAYOUT_MIN_GAP_SECS);
        let area = chart_area(80);
        let layouts = layout_candles(area, &bars, TimeRange::M1);
        let (body_w, gap, _) = fit_candle_body_and_gap_index(area.width, bars.len());
        let stride = body_w + gap;
        assert_eq!(layouts[1].body_left - layouts[0].body_left, stride);
    }

    /// **1D** / **1W** use equal index spacing with fixed body width + gap.
    #[test]
    fn layout_candles_d1_index_stride() {
        let area = chart_area(80);
        let bars = intraday_bars(40);
        let layouts = layout_candles(area, &bars, TimeRange::D1);
        assert_eq!(layouts.len(), 40);
        let (body_w, gap, _) = fit_candle_body_and_gap_index(area.width, bars.len());
        let stride = body_w + gap;
        assert_index_candle_layout_invariants(&layouts, area);
        for pair in layouts.windows(2) {
            assert_eq!(pair[1].body_left, pair[0].body_left + stride);
        }
    }

    #[test]
    fn layout_candles_monotonic() {
        let cases = [
            (60u16, 20usize, TimeRange::M1),
            (80, 22, TimeRange::M1),
            (120, 40, TimeRange::Y1),
        ];
        for (w, n, tr) in cases {
            let area = chart_area(w);
            let bars = daily_bars(n, 1);
            let layouts = layout_candles(area, &bars, tr);
            assert_eq!(layouts.len(), n);
            assert_index_candle_layout_invariants(&layouts, area);
            for pair in layouts.windows(2) {
                assert!(pair[1].body_left > pair[0].body_left);
            }
        }
    }

    #[test]
    fn layout_candles_dense_when_more_bars_than_width() {
        let w = 10u16;
        let area = chart_area(w);
        let bars = intraday_bars(25);
        let layouts = layout_candles(area, &bars, TimeRange::D1);
        assert_eq!(layouts.len(), 25);
        assert_dense_candle_layout_invariants(&layouts, area);
    }

    fn assert_candle_layout_bounds_and_wick(layouts: &[CandleBarLayout], area: Rect) {
        let left = area.left();
        let right = area.right().saturating_sub(1);
        for layout in layouts {
            assert!(layout.body_left <= layout.wick_x());
            assert!(layout.wick_x() <= layout.body_right);
            assert_eq!(layout.wick_x(), layout.body_left + layout.body_w() / 2);
            assert!(layout.body_left >= left && layout.body_right <= right);
        }
        if !layouts.is_empty() {
            assert!(layouts[0].body_left >= left);
            assert!(layouts.last().expect("non-empty").body_right <= right);
        }
    }

    fn assert_index_candle_layout_invariants(layouts: &[CandleBarLayout], area: Rect) {
        assert_candle_layout_bounds_and_wick(layouts, area);
        for layout in layouts {
            assert!((1..=CANDLE_BODY_W_MAX).contains(&layout.body_w()));
        }
    }

    fn assert_dense_candle_layout_invariants(layouts: &[CandleBarLayout], area: Rect) {
        assert_candle_layout_bounds_and_wick(layouts, area);
        for layout in layouts {
            assert!(layout.body_w() >= 1);
        }
    }

    fn candle_fixture_bars(n: usize) -> Vec<HistoricalData> {
        daily_bars(n, 1)
    }

    fn render_candlestick_snapshot(width: u16, height: u16, bars: usize) -> String {
        use crate::app::snapshot_test_util::{buffer_snapshot_string, render_to_buffer};
        use crate::config::theme::ThemePreset;

        let data = candle_fixture_bars(bars);
        let (min_y, max_y) = price_bounds(&data).expect("fixture bounds");
        let theme = ResolvedTheme::from_palette(ThemePreset::Dark.base_rgb());
        let area = Rect::new(0, 0, width, height);
        let layouts = layout_candles(area, &data, TimeRange::M1);
        let chart = CandlestickChart {
            data: &data,
            layouts: &layouts,
            min_y,
            max_y,
            theme,
        };
        let buf = render_to_buffer(width, height, |f| {
            f.render_widget(chart, area);
        });
        buffer_snapshot_string(&buf)
    }

    /// Issue #190 / SPEC §63.5 — candlestick density at 80×24.
    #[test]
    fn candlestick_density_80x24() {
        insta::assert_snapshot!(
            "candlestick_density_80x24",
            render_candlestick_snapshot(80, 24, 20)
        );
    }

    /// Issue #190 / SPEC §63.5 — candlestick density at 120×40.
    #[test]
    fn candlestick_density_120x40() {
        insta::assert_snapshot!(
            "candlestick_density_120x40",
            render_candlestick_snapshot(120, 40, 40)
        );
    }

    fn candle_layout_key(
        area: Rect,
        viewport: ChartViewport,
        bars_len: usize,
        time_range: TimeRange,
        series_stamp: u64,
    ) -> ChartCandleLayoutKey {
        ChartCandleLayoutKey {
            area,
            viewport,
            bars_len,
            time_range,
            series_stamp,
        }
    }

    fn app_with_candle_hist(bars: usize) -> App {
        let mut app = App::new();
        app.chart_mode = ChartDisplayMode::Candlestick;
        app.symbol = "AAPL".to_string();
        app.time_range = TimeRange::Y1;
        app.historical_data = Some(hist("AAPL", bars));
        app.chart_viewport = ChartViewport::full(bars);
        app
    }

    #[test]
    fn candle_layout_cache_key_eq() {
        let area = chart_area(80);
        let vp = ChartViewport::full(20);
        let k1 = candle_layout_key(area, vp, 20, TimeRange::Y1, 1);
        let k2 = candle_layout_key(area, vp, 20, TimeRange::Y1, 1);
        assert_eq!(k1, k2);
        assert_ne!(
            k1,
            candle_layout_key(Rect { width: 120, ..area }, vp, 20, TimeRange::Y1, 1)
        );
        assert_ne!(
            k1,
            candle_layout_key(
                area,
                ChartViewport { start: 5, end: 15 },
                20,
                TimeRange::Y1,
                1
            )
        );
        assert_ne!(k1, candle_layout_key(area, vp, 15, TimeRange::Y1, 1));
        assert_ne!(k1, candle_layout_key(area, vp, 20, TimeRange::M1, 1));
        assert_ne!(k1, candle_layout_key(area, vp, 20, TimeRange::Y1, 2));
    }

    #[test]
    fn candle_layout_cache_hit_returns_same_slice() {
        use crate::app::app::{
            test_candle_layout_build_count, test_reset_candle_layout_build_counter,
        };

        test_reset_candle_layout_build_counter();
        assert_eq!(test_candle_layout_build_count(), 0);
        let mut app = app_with_candle_hist(20);
        let area = chart_area(80);
        let price_area = charts_price_area(area, "AAPL Y1", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 1);
        let ptr1 = app.chart_candle_layout.as_ref().unwrap().layouts.as_ptr();
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 1);
        let ptr2 = app.chart_candle_layout.as_ref().unwrap().layouts.as_ptr();
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn candle_layout_cache_invalidates_on_viewport_change() {
        use crate::app::app::{
            test_candle_layout_build_count, test_reset_candle_layout_build_counter,
        };

        test_reset_candle_layout_build_counter();
        let mut app = app_with_candle_hist(20);
        let price_area =
            charts_price_area(chart_area(80), "AAPL", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 1);
        app.chart_viewport = ChartViewport { start: 5, end: 15 };
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 2);
    }

    #[test]
    fn candle_layout_cache_invalidates_on_bars_len() {
        use crate::app::app::{
            test_candle_layout_build_count, test_reset_candle_layout_build_counter,
        };

        test_reset_candle_layout_build_counter();
        let mut app = app_with_candle_hist(20);
        let price_area =
            charts_price_area(chart_area(80), "AAPL", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 1);
        app.historical_data = Some(hist("AAPL", 25));
        app.chart_viewport = ChartViewport::full(25);
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 2);
    }

    #[test]
    fn candle_layout_cache_invalidates_on_time_range() {
        use crate::app::app::{
            test_candle_layout_build_count, test_reset_candle_layout_build_counter,
        };

        test_reset_candle_layout_build_counter();
        let mut app = app_with_candle_hist(20);
        let price_area =
            charts_price_area(chart_area(80), "AAPL", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 1);
        app.time_range = TimeRange::M1;
        app.prepare_charts_draw_cache(price_area);
        assert_eq!(test_candle_layout_build_count(), 2);
    }

    #[test]
    fn candle_layout_cache_invalidates_on_area_resize() {
        use crate::app::app::{
            test_candle_layout_build_count, test_reset_candle_layout_build_counter,
        };

        test_reset_candle_layout_build_counter();
        let mut app = app_with_candle_hist(20);
        let price_area_80 =
            charts_price_area(chart_area(80), "AAPL", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area_80);
        assert_eq!(test_candle_layout_build_count(), 1);
        let price_area_120 =
            charts_price_area(chart_area(120), "AAPL", ChartIndicatorToggles::default());
        app.prepare_charts_draw_cache(price_area_120);
        assert_eq!(test_candle_layout_build_count(), 2);
    }

    #[test]
    fn charts_price_area_pure() {
        let area = Rect::new(0, 0, 80, 24);
        let title = "AAPL Y1 │ 1-4 c + - h l 0";
        let cases = [
            (ChartIndicatorToggles::default(), "off"),
            (
                ChartIndicatorToggles {
                    rsi_14: true,
                    ..ChartIndicatorToggles::default()
                },
                "rsi",
            ),
            (
                ChartIndicatorToggles {
                    macd: true,
                    ..ChartIndicatorToggles::default()
                },
                "macd",
            ),
            (
                ChartIndicatorToggles {
                    rsi_14: true,
                    macd: true,
                    ..ChartIndicatorToggles::default()
                },
                "rsi+macd",
            ),
        ];
        for (indicators, _label) in cases {
            let got = charts_price_area(area, title, indicators);
            let inner = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .inner(area);
            let expected = if indicators.needs_subpane() {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(55), Constraint::Min(4)])
                    .split(inner)[0]
            } else {
                inner
            };
            assert_eq!(got, expected, "mismatch for {_label}");
        }
    }
}
