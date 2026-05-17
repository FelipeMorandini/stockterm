use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::layout::Layout;
use super::theme::Theme;
use std::collections::HashMap;
use crate::models::alerts::Alert;
use crate::models::portfolio::PortfolioItem;
use thiserror::Error;

/// Market data backend. Yahoo is the default (no API key). Polygon requires `api_key` / `STOCKTERM_API_KEY`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketProviderKind {
    #[default]
    Yahoo,
    Polygon,
}

/// `~/.stockterm.json` — persisted preferences, portfolio, watchlist, alerts, and session hints.
///
/// | Field | Role |
/// |-------|------|
/// | `portfolio` | Holdings (symbol, shares, cost). Default: empty. |
/// | `watchlist` | Stock View symbols (uppercase). Default: empty. |
/// | `refresh_rate` | Quote poll interval (seconds; app may enforce a minimum). Default: `0` → app default. |
/// | `api_key` | Polygon API key (optional if `STOCKTERM_API_KEY` is set). Default: empty. |
/// | `alerts` | Price alerts. Default: empty. |
/// | `default_symbol` | Startup symbol when `watchlist` is empty. Default: empty → app uses `AAPL`. |
/// | `theme` | Optional theme preset + hex overrides (see §21). Default: `null`. |
/// | `provider` | `yahoo` or `polygon`. Default: `yahoo`. |
/// | `notifications_enabled` | Desktop toasts for alerts. Default: `true`. |
/// | `last_tab` | Last focused tab id (`stock_view`, `portfolio`, …). Default: omitted. |
/// | `last_symbol` | Last active ticker (uppercase) when `watchlist` was empty at launch. Default: omitted. |
/// | `keymap` | Optional chord → action overrides (see **README** “Keymap” and [`keymap`](crate::config::keymap)). Default: omitted → built-in defaults. |
/// | `layout` | Shell chrome + pane splits (see §31 / [`layout`](crate::config::layout)). Default: omitted → built-in defaults. |
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub portfolio: Vec<PortfolioItem>,
    /// Symbols to show in the Stock View watchlist table (uppercase tickers).
    #[serde(default)]
    pub watchlist: Vec<String>,
    pub refresh_rate: u64,
    pub api_key: String,
    pub alerts: Vec<Alert>,
    pub default_symbol: String,
    pub theme: Option<Theme>,
    /// When `Polygon`, [`effective_api_key`](Config::effective_api_key) must be non-empty for API calls.
    #[serde(default)]
    pub provider: MarketProviderKind,
    /// Desktop toast when a price alert fires (bell always rings per SPEC §18.5).
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
    /// Last focused tab (`stock_view`, `portfolio`, `alerts`, `search`, `news`, `charts`, `settings`).
    #[serde(default)]
    pub last_tab: Option<String>,
    /// Last active symbol (normalized) when restoring session; used when `watchlist` is empty (Issue #19 / §22).
    #[serde(default)]
    pub last_symbol: Option<String>,
    /// Optional keyboard overrides: JSON object mapping **chord** string → **action** name (PascalCase).
    #[serde(default)]
    pub keymap: Option<HashMap<String, String>>,
    /// Layout visibility and pane sizing (Issue #15 / §31).
    #[serde(default)]
    pub layout: Layout,
}

fn default_notifications_enabled() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            portfolio: Vec::new(),
            watchlist: Vec::new(),
            refresh_rate: 0,
            api_key: String::new(),
            alerts: Vec::new(),
            default_symbol: String::new(),
            theme: None,
            provider: MarketProviderKind::default(),
            notifications_enabled: default_notifications_enabled(),
            last_tab: None,
            last_symbol: None,
            keymap: None,
            layout: Layout::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("home directory not found")]
    NoHomeDir,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Read `path` as JSON [`Config`]. Missing file → [`Config::default`]; same rules as [`Config::try_load`] after path resolution.
fn load_config_from_path(path: &Path) -> Result<Config, ConfigError> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(ConfigError::Io(e)),
        Ok(s) => serde_json::from_str(&s).map_err(ConfigError::Serde),
    }
}

impl Config {
    /// API key resolution (used for **Polygon** only):
    /// 1. Non-empty `api_key` from config file (`~/.stockterm.json`).
    /// 2. Else non-empty `STOCKTERM_API_KEY` environment variable.
    /// 3. Else empty string (Polygon calls fail until configured).
    pub fn effective_api_key(&self) -> Cow<'_, str> {
        if !self.api_key.is_empty() {
            return Cow::Borrowed(self.api_key.as_str());
        }
        match std::env::var("STOCKTERM_API_KEY") {
            Ok(s) if !s.is_empty() => Cow::Owned(s),
            _ => Cow::Borrowed(""),
        }
    }

    /// Load config from disk, or [`Config::default`] on any error.
    ///
    /// **Prefer [`try_load`](Self::try_load)** for interactive applications: failures are invisible
    /// here (silent reset to defaults). [`crate::app::App::new`] uses `try_load` and surfaces
    /// errors via the startup banner (Issue #35 / SPEC §22.7.2).
    pub fn load() -> Self {
        Self::try_load().unwrap_or_default()
    }

    /// Infallible load; same as [`load`](Self::load).
    pub fn load_or_default() -> Self {
        Self::try_load().unwrap_or_default()
    }

    pub fn try_load() -> Result<Self, ConfigError> {
        let path = match Self::config_file_path() {
            Ok(p) => p,
            Err(ConfigError::NoHomeDir) => return Ok(Config::default()),
            Err(e) => return Err(e),
        };
        load_config_from_path(&path)
    }

    /// Persist config. Errors are dropped (TUI has no logger); use [`try_save`](Self::try_save) to handle them.
    pub fn save(&self) {
        let _ = self.try_save();
    }

    pub fn try_save(&self) -> Result<(), ConfigError> {
        let path = Self::config_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let config_str = serde_json::to_string_pretty(self).map_err(ConfigError::Serde)?;
        fs::write(path, config_str).map_err(ConfigError::Io)?;
        Ok(())
    }

    fn config_file_path() -> Result<PathBuf, ConfigError> {
        let mut path = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        path.push(".stockterm.json");
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn effective_api_key_prefers_config_file_value() {
        let c = Config {
            api_key: "from-config".to_string(),
            ..Default::default()
        };
        assert_eq!(c.effective_api_key().as_ref(), "from-config");
    }

    #[test]
    fn default_provider_is_yahoo() {
        let c = Config::default();
        assert_eq!(c.provider, MarketProviderKind::Yahoo);
    }

    #[test]
    fn serde_notifications_enabled_defaults_when_omitted() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert!(c.notifications_enabled);
    }

    #[test]
    fn serde_provider_lowercase() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"polygon"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert_eq!(c.provider, MarketProviderKind::Polygon);
    }

    #[test]
    fn serde_last_tab_last_symbol_default_when_omitted() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert!(c.last_tab.is_none());
        assert!(c.last_symbol.is_none());
    }

    #[test]
    fn serde_keymap_defaults_when_omitted() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert!(c.keymap.is_none());
    }

    #[test]
    fn serde_keymap_parses_object() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo","keymap":{"colon":"Quit"}}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert_eq!(
            c.keymap.as_ref().unwrap().get("colon").map(String::as_str),
            Some("Quit")
        );
    }

    #[test]
    fn load_config_from_path_invalid_json_returns_serde_error() {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/_stockterm_corrupt_cfg_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("corrupt.json");
        fs::write(&path, "{ not valid json").expect("write corrupt config");
        let res = super::load_config_from_path(&path);
        let _ = fs::remove_dir_all(&dir);
        assert!(
            matches!(res, Err(ConfigError::Serde(_))),
            "expected Serde error, got {res:?}"
        );
    }

}
