//! Per-pane renderers for the Dashboard tab (Issue #24 / §70).

use crate::app::styles::ResolvedTheme;
use crate::app::ui::{draw_watchlist_pane_readonly, truncate_visual};
use crate::app::App;
use crate::models::dashboard::{DashboardPane, DashboardPaneKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::atomic::{AtomicBool, Ordering};

const STUB_KIND_SLOTS: usize = 7;

static STUB_WARNED: [AtomicBool; STUB_KIND_SLOTS] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

fn stub_kind_slot(kind: DashboardPaneKind) -> usize {
    match kind {
        DashboardPaneKind::Watchlist => 0,
        DashboardPaneKind::StockDetail => 1,
        DashboardPaneKind::Chart => 2,
        DashboardPaneKind::News => 3,
        DashboardPaneKind::Portfolio => 4,
        DashboardPaneKind::AlertsList => 5,
        DashboardPaneKind::IndicatorSummary => 6,
    }
}

fn warn_stub_kind_once(kind: DashboardPaneKind) {
    let slot = stub_kind_slot(kind);
    if !STUB_WARNED[slot].swap(true, Ordering::Relaxed) {
        tracing::warn!(
            kind = ?kind,
            "dashboard pane kind not implemented in Phase A (§70.9)"
        );
    }
}

/// Draw one dashboard pane into `area`.
pub fn render_dashboard_pane(
    f: &mut Frame,
    app: &mut App,
    pane: &DashboardPane,
    area: Rect,
    rt: ResolvedTheme,
) {
    let error_line = app.error_message();
    let (body_area, footer_area) = split_pane_body_footer(area, error_line.is_some());

    match pane.kind {
        DashboardPaneKind::Watchlist => {
            draw_watchlist_pane_readonly(f, app, body_area, rt, pane.title.as_deref());
        }
        DashboardPaneKind::StockDetail
        | DashboardPaneKind::Chart
        | DashboardPaneKind::News
        | DashboardPaneKind::Portfolio
        | DashboardPaneKind::AlertsList
        | DashboardPaneKind::IndicatorSummary => {
            warn_stub_kind_once(pane.kind);
            draw_stub_pane(f, body_area, rt, pane, "Pane not available (Phase B/C)");
        }
    }

    if let (Some(msg), Some(footer)) = (error_line.as_deref(), footer_area) {
        draw_pane_error_footer(f, footer, rt, msg);
    }
}

fn split_pane_body_footer(area: Rect, reserve_footer: bool) -> (Rect, Option<Rect>) {
    if !reserve_footer || area.height < 4 {
        return (area, None);
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    (chunks[0], Some(chunks[1]))
}

fn draw_stub_pane(f: &mut Frame, area: Rect, rt: ResolvedTheme, pane: &DashboardPane, msg: &str) {
    let title = pane
        .title
        .as_deref()
        .unwrap_or(match pane.kind {
            DashboardPaneKind::StockDetail => "Detail",
            DashboardPaneKind::Chart => "Chart",
            DashboardPaneKind::News => "News",
            DashboardPaneKind::Portfolio => "Portfolio",
            DashboardPaneKind::AlertsList => "Alerts",
            DashboardPaneKind::IndicatorSummary => "Indicators",
            DashboardPaneKind::Watchlist => "Watchlist",
        });
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let text = vec![Line::from(Span::styled(msg, rt.fg_border()))];
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_pane_error_footer(f: &mut Frame, area: Rect, rt: ResolvedTheme, err: &str) {
    let line = Line::from(Span::styled(
        truncate_visual(err, area.width.saturating_sub(2) as usize),
        rt.fg_negative(),
    ));
    f.render_widget(Paragraph::new(line), area);
}
