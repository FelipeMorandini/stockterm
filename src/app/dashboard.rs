//! Dashboard tab grid layout and draw dispatch (Issue #24 / §70).

use crate::app::dashboard_panes::render_dashboard_pane;
use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::models::dashboard::DashboardDefinition;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// How [`Config::active_dashboard`] resolves at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveDashboardResolve {
    /// `active_dashboard` omitted or empty.
    Unconfigured,
    /// Name set but no matching `dashboards[].name`.
    Unknown {
        name: String,
    },
    Ready(DashboardDefinition),
}

/// Resolve the active dashboard from config (Issue #24 / §70).
pub fn resolve_active_dashboard(config: &crate::config::Config) -> ActiveDashboardResolve {
    let Some(name) = config
        .active_dashboard
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return ActiveDashboardResolve::Unconfigured;
    };
    match config.dashboards.iter().find(|d| d.name == name) {
        Some(def) => ActiveDashboardResolve::Ready(def.clone()),
        None => ActiveDashboardResolve::Unknown {
            name: name.to_string(),
        },
    }
}

/// Cache key for dashboard pane pixel layout (§70 audit — off 60fps path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardLayoutKey {
    pub active_name: String,
    pub area_x: u16,
    pub area_y: u16,
    pub area_w: u16,
    pub area_h: u16,
    pub rows: u8,
    pub cols: u8,
    pub pane_count: usize,
}

/// Cached pane id → terminal rect for one dashboard draw.
#[derive(Debug, Clone)]
pub struct DashboardLayoutCache {
    pub key: DashboardLayoutKey,
    pub pane_rects: HashMap<String, Rect>,
}

/// Pixel area for one grid cell within `area`, or `None` when the grid cannot be laid out.
pub fn dashboard_pane_rect(
    def: &DashboardDefinition,
    pane_row: u8,
    pane_col: u8,
    area: Rect,
) -> Option<Rect> {
    if def.rows == 0 || def.cols == 0 || area.width < 2 || area.height < 2 {
        return None;
    }
    let row_constraints: Vec<Constraint> = (0..def.rows)
        .map(|_| Constraint::Ratio(1, u32::from(def.rows)))
        .collect();
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);
    if pane_row as usize >= row_chunks.len() {
        return None;
    }
    let col_constraints: Vec<Constraint> = (0..def.cols)
        .map(|_| Constraint::Ratio(1, u32::from(def.cols)))
        .collect();
    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(row_chunks[pane_row as usize]);
    if pane_col as usize >= col_chunks.len() {
        return None;
    }
    Some(col_chunks[pane_col as usize])
}

/// Union of grid cells for a pane span.
pub fn dashboard_pane_area(
    def: &DashboardDefinition,
    pane: &crate::models::dashboard::DashboardPane,
    area: Rect,
) -> Option<Rect> {
    let mut top = u16::MAX;
    let mut left = u16::MAX;
    let mut bottom = 0u16;
    let mut right = 0u16;
    let row_end = pane.row.saturating_add(pane.row_span);
    let col_end = pane.col.saturating_add(pane.col_span);
    for r in pane.row..row_end {
        for c in pane.col..col_end {
            let cell = dashboard_pane_rect(def, r, c, area)?;
            top = top.min(cell.top());
            left = left.min(cell.left());
            bottom = bottom.max(cell.bottom());
            right = right.max(cell.right());
        }
    }
    if right > left && bottom > top {
        Some(Rect {
            x: left,
            y: top,
            width: right.saturating_sub(left),
            height: bottom.saturating_sub(top),
        })
    } else {
        None
    }
}

/// Map each pane id → layout rect (used by layout cache rebuild).
pub fn dashboard_grid_chunks(def: &DashboardDefinition, area: Rect) -> HashMap<String, Rect> {
    let mut out = HashMap::new();
    for pane in &def.panes {
        if let Some(rect) = dashboard_pane_area(def, pane, area) {
            if rect.width > 0 && rect.height > 0 {
                out.insert(pane.id.clone(), rect);
            }
        }
    }
    out
}

fn dashboard_layout_key(
    def: &DashboardDefinition,
    area: Rect,
    active_name: &str,
) -> DashboardLayoutKey {
    DashboardLayoutKey {
        active_name: active_name.to_string(),
        area_x: area.x,
        area_y: area.y,
        area_w: area.width,
        area_h: area.height,
        rows: def.rows,
        cols: def.cols,
        pane_count: def.panes.len(),
    }
}

impl App {
    /// Rebuild dashboard pane rects when terminal area or definition changes (§70 audit).
    pub(crate) fn prepare_dashboard_layout_cache(&mut self, def: &DashboardDefinition, area: Rect) {
        let active_name = self.config.active_dashboard.as_deref().unwrap_or("");
        let key = dashboard_layout_key(def, area, active_name);
        if self
            .dashboard_layout_cache
            .as_ref()
            .is_some_and(|c| c.key == key)
        {
            return;
        }
        let pane_rects = dashboard_grid_chunks(def, area);
        self.dashboard_layout_cache = Some(DashboardLayoutCache { key, pane_rects });
    }

    /// Cached rect for a dashboard pane id after [`Self::prepare_dashboard_layout_cache`].
    pub(crate) fn dashboard_pane_rect_cached(&self, pane_id: &str) -> Option<Rect> {
        self.dashboard_layout_cache
            .as_ref()?
            .pane_rects
            .get(pane_id)
            .copied()
    }
}

/// Draw the active dashboard or an empty-state message.
pub fn draw_dashboard(f: &mut Frame, app: &mut App, area: Rect, rt: ResolvedTheme) {
    match resolve_active_dashboard(&app.config) {
        ActiveDashboardResolve::Unconfigured => {
            draw_dashboard_unconfigured(f, area, rt);
        }
        ActiveDashboardResolve::Unknown { name } => {
            draw_dashboard_message(
                f,
                area,
                rt,
                &format!(
                    "Unknown dashboard \"{name}\" — check active_dashboard in ~/.stockterm.json"
                ),
            );
        }
        ActiveDashboardResolve::Ready(def) => {
            if app.dashboard_pane_draw_cache.is_empty() && !def.panes.is_empty() {
                crate::app::dashboard_display::rebuild_dashboard_display_strings(app);
            }
            if def.panes.is_empty() {
                draw_dashboard_message(
                    f,
                    area,
                    rt,
                    "Dashboard has no panes after config normalization",
                );
                return;
            }
            if area.width < 4 || area.height < 4 {
                draw_dashboard_message(f, area, rt, "Terminal too small for dashboard");
                return;
            }
            app.prepare_dashboard_layout_cache(&def, area);
            for pane in &def.panes {
                let Some(pane_area) = app.dashboard_pane_rect_cached(&pane.id) else {
                    continue;
                };
                if pane_area.width < 2 || pane_area.height < 2 {
                    continue;
                }
                render_dashboard_pane(f, app, pane, pane_area, rt);
            }
        }
    }
}

fn draw_dashboard_unconfigured(f: &mut Frame, area: Rect, rt: ResolvedTheme) {
    draw_dashboard_message(
        f,
        area,
        rt,
        "No dashboard configured. Set active_dashboard and dashboards in ~/.stockterm.json (see README).",
    );
}

fn draw_dashboard_message(f: &mut Frame, area: Rect, rt: ResolvedTheme, msg: &str) {
    let block = Block::default()
        .title("Dashboard")
        .borders(Borders::ALL)
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let text = vec![Line::from(Span::styled(msg, rt.fg_border()))];
    f.render_widget(Paragraph::new(text).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::snapshot_test_util::{
        buffer_snapshot_string, full_area, render_to_buffer, SNAPSHOT_HEIGHT, SNAPSHOT_WIDTH,
    };
    use crate::app::App;
    use crate::app::Tab;
    use crate::models::dashboard::{preset_dual_watchlist, preset_market_overview};

    #[test]
    fn dashboard_grid_chunks_dual_watchlist_side_by_side() {
        let def = preset_dual_watchlist();
        let area = Rect::new(0, 0, 80, 24);
        let chunks = dashboard_grid_chunks(&def, area);
        assert_eq!(chunks.len(), 2);
        let left = chunks.get("wl_left").expect("left pane");
        let right = chunks.get("wl_right").expect("right pane");
        assert!(right.x >= left.right());
        assert_eq!(left.y, right.y);
    }

    #[test]
    fn resolve_unknown_dashboard_name() {
        let cfg = crate::config::Config {
            active_dashboard: Some("missing".into()),
            ..Default::default()
        };
        assert_eq!(
            resolve_active_dashboard(&cfg),
            ActiveDashboardResolve::Unknown {
                name: "missing".into()
            }
        );
    }

    #[test]
    fn snapshot_dashboard_dual_watchlist() {
        let mut app = App::new();
        app.config.active_dashboard = Some("dual_watchlist".into());
        app.config.dashboards = vec![preset_dual_watchlist()];
        app.watchlist = vec!["AAPL".into(), "MSFT".into()];
        app.rebuild_table_filter_caches();
        app.active_tab = Tab::Dashboard;

        let buf = render_to_buffer(SNAPSHOT_WIDTH, SNAPSHOT_HEIGHT, |f| {
            let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
            draw_dashboard(f, &mut app, full_area(SNAPSHOT_WIDTH, SNAPSHOT_HEIGHT), rt);
        });
        insta::assert_snapshot!("dashboard_dual_watchlist", buffer_snapshot_string(&buf));
    }

    #[test]
    fn snapshot_dashboard_market_overview() {
        let mut app = App::new();
        app.config.active_dashboard = Some("market_overview".into());
        app.config.dashboards = vec![preset_market_overview()];
        app.symbol = "AAPL".into();
        app.watchlist = vec!["AAPL".into(), "MSFT".into()];
        app.portfolio = vec![crate::models::portfolio::PortfolioItem::new(
            "AAPL".into(),
            10.0,
            150.0,
        )];
        if let Some(row) = app.portfolio.first_mut() {
            row.current_price = Some(180.0);
        }
        app.rebuild_table_filter_caches();
        app.active_tab = Tab::Dashboard;

        let buf = render_to_buffer(120, 40, |f| {
            let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
            draw_dashboard(f, &mut app, full_area(120, 40), rt);
        });
        insta::assert_snapshot!("dashboard_market_overview", buffer_snapshot_string(&buf));
    }

    fn sample_historical(bars: usize) -> crate::models::historical::HistoricalResponse {
        use crate::models::historical::HistoricalData;
        crate::models::historical::HistoricalResponse {
            ticker: "AAPL".into(),
            results: (0..bars)
                .map(|i| HistoricalData {
                    t: (i as u64) * 86_400_000,
                    o: 100.0,
                    h: 101.0,
                    l: 99.0,
                    c: 100.0 + i as f64 * 0.1,
                    v: 1_000.0,
                    n: None,
                    vw: 0.0,
                })
                .collect(),
            status: "OK".into(),
            ..Default::default()
        }
    }

    #[test]
    fn snapshot_dashboard_chart_pane_80x24() {
        use crate::app::charts::{draw_chart_pane_in, ChartDisplayMode};
        use crate::models::dashboard::{
            DashboardDefinition, DashboardPane, DashboardPaneKind, DashboardPaneOptions,
        };
        use crate::models::time_range::TimeRange;

        let mut app = App::new();
        app.symbol = "AAPL".into();
        // Pin session chart state so snapshots match CI (no `~/.stockterm.json` dependency).
        app.time_range = TimeRange::Y1;
        app.chart_mode = ChartDisplayMode::Candlestick;
        app.config.active_dashboard = Some("chart_only".into());
        app.config.dashboards = vec![DashboardDefinition {
            name: "chart_only".into(),
            rows: 1,
            cols: 1,
            panes: vec![DashboardPane {
                id: "chart".into(),
                kind: DashboardPaneKind::Chart,
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: None,
                options: DashboardPaneOptions::default(),
            }],
        }];
        app.historical_data = Some(sample_historical(20));
        app.chart_viewport = crate::app::charts::ChartViewport::full(20);
        crate::app::dashboard_display::rebuild_dashboard_display_strings(&mut app);

        let opts = DashboardPaneOptions::default();
        let block_title = app
            .dashboard_pane_draw_cache
            .get("chart")
            .map(|e| e.block_title.clone())
            .unwrap_or_else(|| app.dashboard_chart_block_title_cache.clone());
        let chart_footer = app.dashboard_chart_footer_cache.clone();
        let buf = render_to_buffer(80, 24, |f| {
            let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
            draw_chart_pane_in(
                f,
                &mut app,
                full_area(80, 24),
                rt,
                &opts,
                &block_title,
                &chart_footer,
            );
        });
        insta::assert_snapshot!("dashboard_chart_pane_80x24", buffer_snapshot_string(&buf));
    }

    #[test]
    fn snapshot_dashboard_chart_symbol_override_placeholder() {
        use crate::app::charts::draw_chart_pane_in;
        use crate::models::dashboard::DashboardPaneOptions;

        let mut app = App::new();
        app.symbol = "AAPL".into();
        crate::app::dashboard_display::rebuild_dashboard_display_strings(&mut app);
        let opts = DashboardPaneOptions {
            symbol: Some("MSFT".into()),
            ..Default::default()
        };
        let chart_footer = app.dashboard_chart_footer_cache.clone();
        let buf = render_to_buffer(80, 24, |f| {
            let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
            draw_chart_pane_in(
                f,
                &mut app,
                full_area(80, 24),
                rt,
                &opts,
                "Chart",
                &chart_footer,
            );
        });
        insta::assert_snapshot!(
            "dashboard_chart_symbol_override_placeholder",
            buffer_snapshot_string(&buf)
        );
    }
}
