//! Precomputed dashboard pane strings (Issue #24 / §70 — off 60fps draw path).

use crate::app::dashboard::dashboard_definition_for_render;
use crate::app::dashboard::ActiveDashboardResolve;
use crate::app::App;
use crate::models::dashboard::{DashboardPane, DashboardPaneKind};

/// Per-pane strings built in the update phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardPaneDrawEntry {
    /// Block title for bordered panes.
    pub block_title: String,
    /// Resolved symbol for [`DashboardPaneKind::StockDetail`] panes.
    pub detail_symbol: String,
}

/// Rebuild dashboard draw strings and stock-detail title cache.
pub fn rebuild_dashboard_display_strings(app: &mut App) {
    app.stock_detail_title_cache = format!("Detail: {}", app.symbol);
    app.dashboard_chart_footer_cache = format!(
        "{} · {}",
        app.time_range.label(),
        app.chart_mode.as_config_str()
    );
    app.dashboard_chart_block_title_cache = format!(
        "{} · {} · {}",
        app.symbol,
        app.time_range.label(),
        app.chart_mode.as_config_str()
    );

    app.dashboard_pane_draw_cache.clear();
    let ActiveDashboardResolve::Ready(def) = dashboard_definition_for_render(app) else {
        return;
    };

    for pane in &def.panes {
        let entry = pane_draw_entry(app, pane);
        app.dashboard_pane_draw_cache.insert(pane.id.clone(), entry);
    }
}

fn pane_draw_entry(app: &App, pane: &DashboardPane) -> DashboardPaneDrawEntry {
    let detail_symbol = resolve_pane_detail_symbol(app, pane);
    let block_title = match pane.kind {
        DashboardPaneKind::Watchlist => pane
            .title
            .clone()
            .unwrap_or_else(|| "Watchlist".to_string()),
        DashboardPaneKind::StockDetail => pane
            .title
            .clone()
            .unwrap_or_else(|| format!("Detail: {detail_symbol}")),
        DashboardPaneKind::News => pane
            .title
            .clone()
            .unwrap_or_else(|| format!("News — {}", app.symbol)),
        DashboardPaneKind::Portfolio => pane
            .title
            .clone()
            .unwrap_or_else(|| "Portfolio".to_string()),
        DashboardPaneKind::AlertsList => pane
            .title
            .clone()
            .unwrap_or_else(|| "Price Alerts".to_string()),
        DashboardPaneKind::Chart => pane
            .title
            .clone()
            .unwrap_or_else(|| app.dashboard_chart_block_title_cache.clone()),
        DashboardPaneKind::IndicatorSummary => pane
            .title
            .clone()
            .unwrap_or_else(|| "Indicators".to_string()),
    };
    DashboardPaneDrawEntry {
        block_title,
        detail_symbol,
    }
}

fn resolve_pane_detail_symbol(app: &App, pane: &DashboardPane) -> String {
    if let Some(ref raw) = pane.options.symbol {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return crate::app::normalize_symbol(trimmed).unwrap_or_else(|| trimmed.to_uppercase());
        }
    }
    app.symbol.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ui::watchlist_quote_for_symbol;
    use crate::app::App;
    use crate::models::dashboard::{DashboardDefinition, DashboardPane, DashboardPaneKind};
    use crate::models::ticker::{TickerResponse, TickerResult};

    #[test]
    fn rebuild_populates_pane_titles() {
        let mut app = App::new();
        app.symbol = "AAPL".into();
        app.config.active_dashboard = Some("t".into());
        app.config.dashboards = vec![DashboardDefinition {
            name: "t".into(),
            rows: 1,
            cols: 1,
            panes: vec![DashboardPane {
                id: "news".into(),
                kind: DashboardPaneKind::News,
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: None,
                options: Default::default(),
            }],
        }];
        rebuild_dashboard_display_strings(&mut app);
        let entry = app
            .dashboard_pane_draw_cache
            .get("news")
            .expect("news pane");
        assert!(entry.block_title.contains("AAPL"));
    }

    #[test]
    fn watchlist_quote_lookup_case_insensitive() {
        let mut app = App::new();
        app.watchlist_quotes.insert(
            "AAPL".into(),
            TickerResponse {
                ticker: "AAPL".into(),
                results: vec![TickerResult {
                    o: 1.0,
                    h: 1.0,
                    l: 1.0,
                    c: 150.0,
                    v: 1.0,
                    t: 0,
                    prev_close: None,
                }],
                status: String::new(),
                error: None,
            },
        );
        assert!(watchlist_quote_for_symbol(&app.watchlist_quotes, "aapl").is_some());
    }
}
