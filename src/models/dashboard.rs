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

impl DashboardPaneKind {
    /// Human-readable label for the in-app editor (Issue #208 / §71).
    pub fn label(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::StockDetail => "stock_detail",
            Self::Chart => "chart",
            Self::News => "news",
            Self::Portfolio => "portfolio",
            Self::AlertsList => "alerts_list",
            Self::IndicatorSummary => "indicator_summary",
        }
    }

    /// Cycle pane kinds in editor (Issue #208 / §71).
    pub fn next(self) -> Self {
        match self {
            Self::Watchlist => Self::StockDetail,
            Self::StockDetail => Self::Chart,
            Self::Chart => Self::News,
            Self::News => Self::Portfolio,
            Self::Portfolio => Self::AlertsList,
            Self::AlertsList => Self::IndicatorSummary,
            Self::IndicatorSummary => Self::Watchlist,
        }
    }
}

/// Editor-time validation error (strict — Issue #208 / §71.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardValidationError {
    EmptyName,
    DuplicateName { name: String },
    GridOutOfRange { rows: u8, cols: u8 },
    EmptyPaneId,
    DuplicatePaneId { id: String },
    PaneOutOfBounds { id: String, reason: &'static str },
    PanesOverlap { a: String, b: String },
    NoPanes,
}

impl std::fmt::Display for DashboardValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(f, "dashboard name is required"),
            Self::DuplicateName { name } => write!(f, "dashboard name \"{name}\" already exists"),
            Self::GridOutOfRange { rows, cols } => {
                write!(f, "grid must be 1–4 rows/cols (got {rows}×{cols})")
            }
            Self::EmptyPaneId => write!(f, "pane id is required"),
            Self::DuplicatePaneId { id } => write!(f, "duplicate pane id \"{id}\""),
            Self::PaneOutOfBounds { id, reason } => {
                write!(f, "pane \"{id}\": {reason}")
            }
            Self::PanesOverlap { a, b } => write!(f, "panes \"{a}\" and \"{b}\" overlap"),
            Self::NoPanes => write!(f, "dashboard needs at least one pane"),
        }
    }
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

/// Clone a built-in preset under a new dashboard name (Issue #208 / §71.4).
pub fn clone_preset_with_name(preset: &DashboardDefinition, new_name: &str) -> DashboardDefinition {
    let mut def = preset.clone();
    def.name = new_name.to_string();
    def
}

/// Unique dashboard name for a new layout (`dashboard_1`, …).
pub fn allocate_dashboard_name(existing: &[DashboardDefinition]) -> String {
    for i in 1..=999u32 {
        let name = format!("dashboard_{i}");
        if !existing.iter().any(|d| d.name == name) {
            return name;
        }
    }
    "dashboard".to_string()
}

/// Unique pane id within a draft (`pane_1`, …).
pub fn allocate_dashboard_pane_id(existing: &[DashboardPane]) -> String {
    for i in 1..=999u32 {
        let id = format!("pane_{i}");
        if !existing.iter().any(|p| p.id == id) {
            return id;
        }
    }
    "pane".to_string()
}

/// Validate a dashboard name is unique in config (editor commit).
pub fn validate_dashboard_name_unique(
    name: &str,
    dashboards: &[DashboardDefinition],
    editing_index: Option<usize>,
) -> Result<(), DashboardValidationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DashboardValidationError::EmptyName);
    }
    for (i, def) in dashboards.iter().enumerate() {
        if Some(i) == editing_index {
            continue;
        }
        if def.name == trimmed {
            return Err(DashboardValidationError::DuplicateName {
                name: trimmed.to_string(),
            });
        }
    }
    Ok(())
}

/// Validate one pane against the grid and siblings (Issue #208 / §71.4).
pub fn validate_dashboard_pane(
    def: &DashboardDefinition,
    pane: &DashboardPane,
    exclude_index: Option<usize>,
) -> Result<(), DashboardValidationError> {
    if pane.id.trim().is_empty() {
        return Err(DashboardValidationError::EmptyPaneId);
    }
    if def.rows < 1 || def.cols < 1 || def.rows > 4 || def.cols > 4 {
        return Err(DashboardValidationError::GridOutOfRange {
            rows: def.rows,
            cols: def.cols,
        });
    }
    if pane.row >= def.rows {
        return Err(DashboardValidationError::PaneOutOfBounds {
            id: pane.id.clone(),
            reason: "row outside grid",
        });
    }
    if pane.col >= def.cols {
        return Err(DashboardValidationError::PaneOutOfBounds {
            id: pane.id.clone(),
            reason: "col outside grid",
        });
    }
    if pane.row_span == 0 || pane.col_span == 0 {
        return Err(DashboardValidationError::PaneOutOfBounds {
            id: pane.id.clone(),
            reason: "span must be at least 1",
        });
    }
    if pane.row.saturating_add(pane.row_span) > def.rows {
        return Err(DashboardValidationError::PaneOutOfBounds {
            id: pane.id.clone(),
            reason: "row span exceeds grid rows",
        });
    }
    if pane.col.saturating_add(pane.col_span) > def.cols {
        return Err(DashboardValidationError::PaneOutOfBounds {
            id: pane.id.clone(),
            reason: "col span exceeds grid cols",
        });
    }
    for (i, other) in def.panes.iter().enumerate() {
        if Some(i) == exclude_index {
            continue;
        }
        if other.id == pane.id {
            return Err(DashboardValidationError::DuplicatePaneId {
                id: pane.id.clone(),
            });
        }
        if dashboard_panes_overlap(pane, other) {
            return Err(DashboardValidationError::PanesOverlap {
                a: pane.id.clone(),
                b: other.id.clone(),
            });
        }
    }
    Ok(())
}

/// Validate a full dashboard before editor save (Issue #208 / §71.4).
pub fn validate_dashboard_definition(
    def: &DashboardDefinition,
) -> Result<(), Vec<DashboardValidationError>> {
    let mut errors = Vec::new();
    if def.name.trim().is_empty() {
        errors.push(DashboardValidationError::EmptyName);
    }
    if def.rows < 1 || def.cols < 1 || def.rows > 4 || def.cols > 4 {
        errors.push(DashboardValidationError::GridOutOfRange {
            rows: def.rows,
            cols: def.cols,
        });
    }
    if def.panes.is_empty() {
        errors.push(DashboardValidationError::NoPanes);
    }
    let mut seen_ids = HashSet::new();
    for (i, pane) in def.panes.iter().enumerate() {
        if pane.id.trim().is_empty() {
            errors.push(DashboardValidationError::EmptyPaneId);
            continue;
        }
        if !seen_ids.insert(pane.id.clone()) {
            errors.push(DashboardValidationError::DuplicatePaneId {
                id: pane.id.clone(),
            });
        }
        if let Err(e) = validate_dashboard_pane(def, pane, Some(i)) {
            if !errors.contains(&e) {
                errors.push(e);
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// First validation message for editor footer.
pub fn format_dashboard_validation_errors(errors: &[DashboardValidationError]) -> String {
    errors
        .first()
        .map(ToString::to_string)
        .unwrap_or_else(|| "invalid dashboard".to_string())
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

    #[test]
    fn validate_dashboard_rejects_overlap() {
        let def = DashboardDefinition {
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
        assert!(validate_dashboard_definition(&def).is_err());
    }

    #[test]
    fn validate_dashboard_accepts_dual_watchlist() {
        let def = preset_dual_watchlist();
        assert!(validate_dashboard_definition(&def).is_ok());
    }

    #[test]
    fn allocate_dashboard_name_skips_existing() {
        let existing = vec![DashboardDefinition {
            name: "dashboard_1".into(),
            rows: 1,
            cols: 1,
            panes: vec![],
        }];
        assert_eq!(allocate_dashboard_name(&existing), "dashboard_2");
    }
}
