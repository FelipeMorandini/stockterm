pub mod alerts;
#[allow(clippy::module_inception)]
pub mod app;
mod app_error;
mod backtest_ui;
pub mod charts;
mod dashboard;
mod dashboard_display;
mod dashboard_editor;
mod dashboard_panes;
pub mod event;
mod fetch_delivery;
mod format;
pub mod handlers;
pub mod keyboard;
mod layout;
mod open_url;
mod options;
pub mod portfolio;
pub mod styles;
mod table_filter;
pub mod ui;
mod watchlist_display;

#[cfg(test)]
mod snapshot_test_util;

pub use self::app::{
    normalize_symbol, AlertAddDialog, AlertAddField, App, FetchDone, PortfolioAddDialog,
    PortfolioAddField, PortfolioDialogKind, SettingsEdit, Tab, SETTINGS_ROW_COUNT,
};
