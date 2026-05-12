#[allow(clippy::module_inception)]
pub mod app;
mod open_url;
pub mod ui;
pub mod event;
pub mod handlers;
pub mod keyboard;
pub mod charts;
pub mod portfolio;
pub mod alerts;

pub use self::app::{
    normalize_symbol, AlertAddDialog, AlertAddField, App, FetchDone, PortfolioAddDialog,
    PortfolioAddField, SettingsEdit, Tab, SETTINGS_ROW_COUNT,
};
