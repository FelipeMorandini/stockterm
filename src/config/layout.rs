//! Issue #15 / [`docs/SPEC.md`](../../docs/SPEC.md) §31 — layout preferences.

use serde::{Deserialize, Serialize};

const STOCK_VIEW_WATCHLIST_PCT_DEFAULT: u8 = 42;
const CHARTS_CHART_PCT_DEFAULT: u8 = 100;
const STOCK_VIEW_WATCHLIST_PCT_MIN: u8 = 20;
const STOCK_VIEW_WATCHLIST_PCT_MAX: u8 = 80;
const CHARTS_CHART_PCT_MIN: u8 = 30;
const CHARTS_CHART_PCT_MAX: u8 = 100;

/// Built-in layout presets (serde snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPreset {
    #[default]
    #[serde(rename = "default")]
    BuiltinDefault,
    Compact,
    Wide,
    ChartFocused,
}

impl LayoutPreset {
    pub const ALL: [LayoutPreset; 4] = [
        LayoutPreset::BuiltinDefault,
        LayoutPreset::Compact,
        LayoutPreset::Wide,
        LayoutPreset::ChartFocused,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LayoutPreset::BuiltinDefault => "default",
            LayoutPreset::Compact => "compact",
            LayoutPreset::Wide => "wide",
            LayoutPreset::ChartFocused => "chart_focused",
        }
    }

    pub fn next(self) -> LayoutPreset {
        let i = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> LayoutPreset {
        let i = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    fn base(self) -> LayoutValues {
        match self {
            LayoutPreset::BuiltinDefault => LayoutValues::builtin_default(),
            LayoutPreset::Compact => LayoutValues {
                show_tab_bar: true,
                show_status_bar: false,
                stock_view_watchlist_pct: 35,
                charts_chart_pct: 100,
            },
            LayoutPreset::Wide => LayoutValues {
                show_tab_bar: true,
                show_status_bar: true,
                stock_view_watchlist_pct: 30,
                charts_chart_pct: 100,
            },
            LayoutPreset::ChartFocused => LayoutValues {
                show_tab_bar: true,
                show_status_bar: true,
                stock_view_watchlist_pct: 35,
                charts_chart_pct: 85,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LayoutValues {
    show_tab_bar: bool,
    show_status_bar: bool,
    stock_view_watchlist_pct: u8,
    charts_chart_pct: u8,
}

impl LayoutValues {
    fn builtin_default() -> Self {
        Self {
            show_tab_bar: true,
            show_status_bar: true,
            stock_view_watchlist_pct: STOCK_VIEW_WATCHLIST_PCT_DEFAULT,
            charts_chart_pct: CHARTS_CHART_PCT_DEFAULT,
        }
    }
}

/// Persisted layout preferences (`~/.stockterm.json` → `layout`).
///
/// Omitted optional fields inherit from `preset` when set, otherwise built-in defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_tab_bar: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_status_bar: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stock_view_watchlist_pct: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charts_chart_pct: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<LayoutPreset>,
}

/// Clamped layout inputs for one frame (Issue #15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedLayout {
    pub show_tab_bar: bool,
    pub show_status_bar: bool,
    pub stock_view_watchlist_pct: u16,
    pub charts_chart_pct: u16,
}

impl Layout {
    pub fn from_preset(preset: LayoutPreset) -> Self {
        let b = preset.base();
        Self {
            show_tab_bar: Some(b.show_tab_bar),
            show_status_bar: Some(b.show_status_bar),
            stock_view_watchlist_pct: Some(b.stock_view_watchlist_pct),
            charts_chart_pct: Some(b.charts_chart_pct),
            preset: Some(preset),
        }
    }

    pub fn effective_preset(&self) -> LayoutPreset {
        self.preset.unwrap_or(LayoutPreset::BuiltinDefault)
    }

    /// Settings row label: preset name, or `"{preset} + overrides"` when resolved layout differs.
    pub fn saved_summary_label(&self) -> String {
        let preset = self.effective_preset();
        let baseline = Layout::from_preset(preset);
        if self.resolve() == baseline.resolve() {
            preset.label().to_string()
        } else {
            format!("{} + overrides", preset.label())
        }
    }

    pub fn resolve(&self) -> ResolvedLayout {
        let base = self
            .preset
            .map(LayoutPreset::base)
            .unwrap_or_else(LayoutValues::builtin_default);
        let show_tab_bar = self.show_tab_bar.unwrap_or(base.show_tab_bar);
        let show_status_bar = self.show_status_bar.unwrap_or(base.show_status_bar);
        let stock = self
            .stock_view_watchlist_pct
            .unwrap_or(base.stock_view_watchlist_pct);
        let charts = self.charts_chart_pct.unwrap_or(base.charts_chart_pct);
        ResolvedLayout {
            show_tab_bar,
            show_status_bar,
            stock_view_watchlist_pct: clamp_stock_view_watchlist_pct(stock),
            charts_chart_pct: clamp_charts_chart_pct(charts),
        }
    }
}

fn clamp_stock_view_watchlist_pct(v: u8) -> u16 {
    u16::from(v.clamp(
        STOCK_VIEW_WATCHLIST_PCT_MIN,
        STOCK_VIEW_WATCHLIST_PCT_MAX,
    ))
}

fn clamp_charts_chart_pct(v: u8) -> u16 {
    u16::from(v.clamp(CHARTS_CHART_PCT_MIN, CHARTS_CHART_PCT_MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_resolves_to_builtin() {
        let r = Layout::default().resolve();
        assert!(r.show_tab_bar);
        assert!(r.show_status_bar);
        assert_eq!(r.stock_view_watchlist_pct, 42);
        assert_eq!(r.charts_chart_pct, 100);
    }

    #[test]
    fn clamp_stock_view_watchlist_pct() {
        let r = Layout {
            stock_view_watchlist_pct: Some(5),
            ..Layout::default()
        }
        .resolve();
        assert_eq!(r.stock_view_watchlist_pct, 20);
        let r = Layout {
            stock_view_watchlist_pct: Some(95),
            ..Layout::default()
        }
        .resolve();
        assert_eq!(r.stock_view_watchlist_pct, 80);
    }

    #[test]
    fn clamp_charts_chart_pct() {
        let r = Layout {
            charts_chart_pct: Some(10),
            ..Layout::default()
        }
        .resolve();
        assert_eq!(r.charts_chart_pct, 30);
        let r = Layout {
            charts_chart_pct: Some(200),
            ..Layout::default()
        }
        .resolve();
        assert_eq!(r.charts_chart_pct, 100);
    }

    #[test]
    fn preset_compact_hides_status_bar() {
        let r = Layout {
            preset: Some(LayoutPreset::Compact),
            ..Layout::default()
        }
        .resolve();
        assert!(!r.show_status_bar);
    }

    #[test]
    fn preset_override_show_status_bar() {
        let r = Layout {
            preset: Some(LayoutPreset::Compact),
            show_status_bar: Some(true),
            ..Layout::default()
        }
        .resolve();
        assert!(r.show_status_bar);
    }

    #[test]
    fn serde_partial_layout_merges_defaults() {
        let j = r#"{"show_status_bar":false}"#;
        let l: Layout = serde_json::from_str(j).unwrap();
        let r = l.resolve();
        assert!(!r.show_status_bar);
        assert!(r.show_tab_bar);
        assert_eq!(r.stock_view_watchlist_pct, 42);
    }

    #[test]
    fn preset_chart_focused_charts_pct() {
        let r = Layout::from_preset(LayoutPreset::ChartFocused).resolve();
        assert_eq!(r.charts_chart_pct, 85);
    }

    #[test]
    fn saved_summary_label_scalar_only() {
        let l = Layout {
            charts_chart_pct: Some(70),
            ..Layout::default()
        };
        assert_eq!(l.saved_summary_label(), "default + overrides");
    }

    #[test]
    fn saved_summary_label_pure_preset() {
        let l = Layout::from_preset(LayoutPreset::Compact);
        assert_eq!(l.saved_summary_label(), "compact");
    }

    #[test]
    fn preview_preset_keeps_scalar_overrides() {
        let l = Layout {
            charts_chart_pct: Some(70),
            ..Layout::default()
        };
        let mut preview = l.clone();
        preview.preset = Some(LayoutPreset::Compact);
        assert_eq!(preview.resolve().charts_chart_pct, 70);
        assert!(!preview.resolve().show_status_bar);
    }

    #[test]
    fn commit_preset_merge_keeps_scalar_overrides() {
        let saved = Layout {
            charts_chart_pct: Some(70),
            ..Layout::default()
        };
        let mut merged = saved.clone();
        merged.preset = Some(LayoutPreset::Compact);
        assert_eq!(merged.resolve().charts_chart_pct, 70);
        assert!(!merged.resolve().show_status_bar);
        assert_eq!(merged.preset, Some(LayoutPreset::Compact));
    }
}
