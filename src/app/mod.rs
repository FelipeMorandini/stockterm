#[allow(clippy::module_inception)]
pub mod app;
pub mod ui;
pub mod event;
pub mod handlers;
pub mod keyboard;
pub mod charts;
pub mod portfolio;
pub mod alerts;

pub use self::app::{normalize_symbol, App, FetchDone, Tab};
