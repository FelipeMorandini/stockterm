//! Composable dashboard pane types for Issue #24 / [`docs/SPEC.md`](../../docs/SPEC.md) §70.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Kind of dashboard pane (serde snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPaneKind {
    Watchlist,
    StockDetail,
    Chart,
    News,
    Portfolio,
    AlertsList,
    /// Charts-tab indicator summary (§46).
    IndicatorSummary,
}

/// Per-pane options (unknown JSON fields ignored via [`Default`] on deserialize).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardPaneOptions {
    /// `Chart` / `IndicatorSummary`: symbol override; default = active symbol.
    pub symbol: Option<String>,
    /// `Chart`: `d1` | `w1` | `m1` | `y1`.
    pub time_range: Option<String>,
    /// `News`: max headline rows (clamped 3..30 at render).
    pub max_rows: Option<u8>,
}

impl DashboardPaneOptions {
    /// True when all optional fields are unset (for `skip_serializing_if`).
    pub fn is_empty(&self) -> bool {
        self.symbol.is_none() && self.time_range.is_none() && self.max_rows.is_none()
    }
}

/// One pane in a dashboard grid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardPane {
    pub id: String,
    pub kind: DashboardPaneKind,
    /// Grid placement: 0-based row index.
    pub row: u8,
    /// Grid placement: 0-based column index.
    pub col: u8,
    pub row_span: u8,
    pub col_span: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub options: DashboardPaneOptions,
}

/// Named dashboard layout stored in config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardDefinition {
    pub name: String,
    /// Grid row count (clamped 1..=4 on load).
    pub rows: u8,
    /// Grid column count (clamped 1..=4 on load).
    pub cols: u8,
    pub panes: Vec<DashboardPane>,
}

/// Issue #24 acceptance fixture: two watchlist panes side-by-side (1×2 grid).
pub fn preset_dual_watchlist() -> DashboardDefinition {
    DashboardDefinition {
        name: "dual_watchlist".to_string(),
        rows: 1,
        cols: 2,
        panes: vec![
            DashboardPane {
                id: "wl_left".to_string(),
                kind: DashboardPaneKind::Watchlist,
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: Some("Watchlist (left)".to_string()),
                options: DashboardPaneOptions::default(),
            },
            DashboardPane {
                id: "wl_right".to_string(),
                kind: DashboardPaneKind::Watchlist,
                row: 0,
                col: 1,
                row_span: 1,
                col_span: 1,
                title: Some("Watchlist (right)".to_string()),
                options: DashboardPaneOptions::default(),
            },
        ],
    }
}

/// 2×2 market overview preset (Issue #24 Phase B).
pub fn preset_market_overview() -> DashboardDefinition {
    DashboardDefinition {
        name: "market_overview".to_string(),
        rows: 2,
        cols: 2,
        panes: vec![
            DashboardPane {
                id: "watchlist".to_string(),
                kind: DashboardPaneKind::Watchlist,
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: Some("Watchlist".to_string()),
                options: DashboardPaneOptions::default(),
            },
            DashboardPane {
                id: "detail".to_string(),
                kind: DashboardPaneKind::StockDetail,
                row: 0,
                col: 1,
                row_span: 1,
                col_span: 1,
                title: Some("Detail".to_string()),
                options: DashboardPaneOptions::default(),
            },
            DashboardPane {
                id: "news".to_string(),
                kind: DashboardPaneKind::News,
                row: 1,
                col: 0,
                row_span: 1,
                col_span: 1,
                title: Some("News".to_string()),
                options: DashboardPaneOptions {
                    max_rows: Some(8),
                    ..DashboardPaneOptions::default()
                },
            },
            DashboardPane {
                id: "portfolio".to_string(),
                kind: DashboardPaneKind::Portfolio,
                row: 1,
                col: 1,
                row_span: 1,
                col_span: 1,
                title: Some("Portfolio".to_string()),
                options: DashboardPaneOptions::default(),
            },
        ],
    }
}

fn clamp_grid_axis(n: u8) -> u8 {
    n.clamp(1, 4)
}

fn clamp_pane_span(span: u8) -> u8 {
    span.max(1)
}

/// Returns true when two pane rectangles overlap on the grid.
pub fn dashboard_panes_overlap(a: &DashboardPane, b: &DashboardPane) -> bool {
    let a_re = a.row.saturating_add(a.row_span);
    let b_re = b.row.saturating_add(b.row_span);
    let a_ce = a.col.saturating_add(a.col_span);
    let b_ce = b.col.saturating_add(b.col_span);
    !(a_re <= b.row || b_re <= a.row || a_ce <= b.col || b_ce <= a.col)
}

/// Clamp grid size, pane spans, dedupe ids, and drop overlapping panes (keep first).
pub fn normalize_dashboard_definition(def: &mut DashboardDefinition) {
    def.rows = clamp_grid_axis(def.rows);
    def.cols = clamp_grid_axis(def.cols);

    let mut seen_ids = HashSet::new();
    let mut kept: Vec<DashboardPane> = Vec::new();

    for mut pane in def.panes.drain(..) {
        if pane.id.is_empty() {
            tracing::warn!(dashboard = %def.name, "dashboard pane dropped: empty id");
            continue;
        }
        if !seen_ids.insert(pane.id.clone()) {
            tracing::warn!(
                dashboard = %def.name,
                pane_id = %pane.id,
                "dashboard pane dropped: duplicate id"
            );
            continue;
        }

        pane.row_span = clamp_pane_span(pane.row_span);
        pane.col_span = clamp_pane_span(pane.col_span);
        if pane.row >= def.rows {
            pane.row = def.rows.saturating_sub(1);
        }
        if pane.col >= def.cols {
            pane.col = def.cols.saturating_sub(1);
        }
        if pane.row.saturating_add(pane.row_span) > def.rows {
            pane.row_span = def.rows.saturating_sub(pane.row);
        }
        if pane.col.saturating_add(pane.col_span) > def.cols {
            pane.col_span = def.cols.saturating_sub(pane.col);
        }
        if pane.row_span == 0 || pane.col_span == 0 {
            tracing::warn!(
                dashboard = %def.name,
                pane_id = %pane.id,
                "dashboard pane dropped: zero span after clamp"
            );
            continue;
        }

        let overlaps = kept.iter().any(|p| dashboard_panes_overlap(p, &pane));
        if overlaps {
            tracing::warn!(
                dashboard = %def.name,
                pane_id = %pane.id,
                "dashboard pane dropped: overlaps an existing pane"
            );
            continue;
        }
        kept.push(pane);
    }
    def.panes = kept;
}

/// Normalize every dashboard in config (Issue #24 / §70.4).
pub fn normalize_dashboards(dashboards: &mut [DashboardDefinition]) {
    for def in dashboards.iter_mut() {
        normalize_dashboard_definition(def);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_dual_watchlist_deserializes() {
        let json = include_str!("../../tests/fixtures/dashboard_dual_watchlist.json");
        let def: DashboardDefinition = serde_json::from_str(json).expect("fixture JSON");
        assert_eq!(def.name, "dual_watchlist");
        assert_eq!(def.rows, 1);
        assert_eq!(def.cols, 2);
        assert_eq!(def.panes.len(), 2);
    }

    #[test]
    fn overlap_drops_second_pane() {
        let mut def = DashboardDefinition {
            name: "t".into(),
            rows: 2,
            cols: 2,
            panes: vec![
                DashboardPane {
                    id: "a".into(),
                    kind: DashboardPaneKind::Watchlist,
                    row: 0,
                    col: 0,
                    row_span: 2,
                    col_span: 2,
                    title: None,
                    options: DashboardPaneOptions::default(),
                },
                DashboardPane {
                    id: "b".into(),
                    kind: DashboardPaneKind::Watchlist,
                    row: 1,
                    col: 1,
                    row_span: 1,
                    col_span: 1,
                    title: None,
                    options: DashboardPaneOptions::default(),
                },
            ],
        };
        normalize_dashboard_definition(&mut def);
        assert_eq!(def.panes.len(), 1);
        assert_eq!(def.panes[0].id, "a");
    }

    #[test]
    fn preset_market_overview_has_four_panes() {
        let def = preset_market_overview();
        assert_eq!(def.rows, 2);
        assert_eq!(def.cols, 2);
        assert_eq!(def.panes.len(), 4);
    }

    #[test]
    fn dashboard_panes_overlap_detects_shared_cell() {
        let a = DashboardPane {
            id: "a".into(),
            kind: DashboardPaneKind::Watchlist,
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            title: None,
            options: DashboardPaneOptions::default(),
        };
        let b = DashboardPane {
            id: "b".into(),
            kind: DashboardPaneKind::Watchlist,
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            title: None,
            options: DashboardPaneOptions::default(),
        };
        assert!(dashboard_panes_overlap(&a, &b));
    }
}
