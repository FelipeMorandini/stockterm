//! Per-pane renderers for the Dashboard tab (Issue #24 / §70).

use crate::app::alerts::draw_alerts_table_in;
use crate::app::charts::{draw_chart_pane_in, draw_indicator_summary_pane_in};
use crate::app::dashboard_display::DashboardPaneDrawEntry;
use crate::app::portfolio::draw_portfolio_table_in;
use crate::app::styles::ResolvedTheme;
use crate::app::ui::{draw_news_list_in, draw_stock_detail_in, draw_watchlist_pane_readonly, truncate_visual};
use crate::app::App;
use crate::models::dashboard::{DashboardPane, DashboardPaneKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

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
    let draw_entry: Option<DashboardPaneDrawEntry> =
        app.dashboard_pane_draw_cache.get(&pane.id).cloned();

    match pane.kind {
        DashboardPaneKind::Watchlist => {
            draw_watchlist_pane_readonly(f, app, body_area, rt, pane.title.as_deref());
        }
        DashboardPaneKind::StockDetail => {
            if let Some(entry) = draw_entry {
                draw_stock_detail_in(
                    f,
                    app,
                    body_area,
                    rt,
                    &entry.block_title,
                    &entry.detail_symbol,
                );
            }
        }
        DashboardPaneKind::News => {
            if let Some(entry) = draw_entry {
                draw_news_list_in(
                    f,
                    app,
                    body_area,
                    rt,
                    pane.options.max_rows,
                    &entry.block_title,
                    false,
                );
            }
        }
        DashboardPaneKind::Portfolio => {
            if let Some(entry) = draw_entry {
                draw_portfolio_table_in(f, app, body_area, rt, &entry.block_title, false);
            }
        }
        DashboardPaneKind::AlertsList => {
            if let Some(entry) = draw_entry {
                draw_alerts_table_in(f, app, body_area, rt, &entry.block_title, false);
            }
        }
        DashboardPaneKind::Chart => {
            let block_title = draw_entry
                .map(|e| e.block_title)
                .unwrap_or_else(|| app.dashboard_chart_block_title_cache.clone());
            let chart_footer = app.dashboard_chart_footer_cache.clone();
            draw_chart_pane_in(
                f,
                app,
                body_area,
                rt,
                &pane.options,
                &block_title,
                &chart_footer,
            );
        }
        DashboardPaneKind::IndicatorSummary => {
            draw_indicator_summary_pane_in(
                f,
                app,
                body_area,
                rt,
                &pane.options,
                pane.title.as_deref(),
            );
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

fn draw_pane_error_footer(f: &mut Frame, area: Rect, rt: ResolvedTheme, err: &str) {
    let line = Line::from(Span::styled(
        truncate_visual(err, area.width.saturating_sub(2) as usize),
        rt.fg_negative(),
    ));
    f.render_widget(Paragraph::new(line), area);
}
