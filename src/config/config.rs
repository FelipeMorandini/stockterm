use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::layout::Layout;
use super::theme::Theme;
use crate::models::alerts::Alert;
use crate::models::backtest::{BacktestConfig, BacktestStrategyParams};
use crate::models::dashboard::{normalize_dashboards, DashboardDefinition};
use crate::models::portfolio::PortfolioItem;
use crate::models::saved_filter::{sanitize_saved_filters, SavedFilter};
use crate::models::symbol::{canonicalize_persisted_symbol_fields, SymbolCanonicalizeReport};
use std::collections::HashMap;
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
/// | `api_key` | Polygon API key stored in JSON only (see [`effective_api_key`](Config::effective_api_key); env is not copied here on load). Default: empty. |
/// | `alerts` | Price alerts. Default: empty. |
/// | `default_symbol` | Startup symbol when `watchlist` is empty. Default: empty → app uses `AAPL`. |
/// | `theme` | Optional theme preset + hex overrides (see §21). Default: `null`. |
/// | `provider` | `yahoo` or `polygon`. Default: `yahoo`. |
/// | `notifications_enabled` | Desktop toasts for alerts. Default: `true`. |
/// | `last_tab` | Last focused tab id (`stock_view`, `portfolio`, …). Default: omitted. |
/// | `last_symbol` | Last active ticker (uppercase) when `watchlist` was empty at launch. Default: omitted. |
/// | `last_time_range` | Last Charts time window (`d1`, `w1`, `m1`, `y1`). Default: omitted. |
/// | `last_chart_mode` | Last Charts display mode (`line`, `candles`). Default: omitted. |
/// | `keymap` | Optional chord → action overrides (see **README** “Keymap” and [`keymap`](crate::config::keymap)). Default: omitted → built-in defaults. |
/// | `layout` | Shell chrome + pane splits (see §31 / [`layout`](crate::config::layout)). Default: omitted → built-in defaults. |
/// | `saved_filters` | Named table filters for Stock View / Portfolio (Issue #194 / §69). Default: empty. |
/// | `dashboards` | Composable dashboard layouts (Issue #24 / §70). Default: empty. |
/// | `active_dashboard` | Name of dashboard to show on **Dashboard** tab. Default: omitted. |
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub portfolio: Vec<PortfolioItem>,
    /// Symbols to show in the Stock View watchlist table (uppercase tickers).
    #[serde(default)]
    pub watchlist: Vec<String>,
    pub refresh_rate: u64,
    /// Polygon API key as persisted in `~/.stockterm.json` (plaintext).
    ///
    /// Runtime resolution for Polygon HTTP uses [`effective_api_key`](Self::effective_api_key):
    /// non-empty file value wins; else non-empty `STOCKTERM_API_KEY`. The env var is never
    /// merged into this field on [`try_load`](Self::try_load) or written by [`try_save`](Self::try_save)
    /// unless the user (or future in-app editor) sets it explicitly (Issue #28 / SPEC §42.2).
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
    /// Last Charts tab time window (`d1` / `w1` / `m1` / `y1`). Invalid or omitted → app default (Issue #180 / §54).
    #[serde(default)]
    pub last_time_range: Option<String>,
    /// Last Charts display mode (`line` / `candles`). Invalid or omitted → app default (Issue #180 / §54).
    #[serde(default)]
    pub last_chart_mode: Option<String>,
    /// Optional keyboard overrides: JSON object mapping **chord** string → **action** name (PascalCase).
    #[serde(default)]
    pub keymap: Option<HashMap<String, String>>,
    /// Layout visibility and pane sizing (Issue #15 / §31).
    #[serde(default)]
    pub layout: Layout,
    /// Backtest simulation parameters (Issue #25 / §47).
    #[serde(default)]
    pub backtest: BacktestConfig,
    /// Active backtest strategy and periods (Issue #25 / §47).
    #[serde(default)]
    pub backtest_strategy: BacktestStrategyParams,
    /// Named substring/regex filters (Issue #194 / §69).
    #[serde(default)]
    pub saved_filters: Vec<SavedFilter>,
    /// User-defined dashboard layouts (Issue #24 / §70).
    #[serde(default)]
    pub dashboards: Vec<DashboardDefinition>,
    /// Active dashboard name; must match an entry in [`Self::dashboards`].
    #[serde(default)]
    pub active_dashboard: Option<String>,
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
            last_time_range: None,
            last_chart_mode: None,
            keymap: None,
            layout: Layout::default(),
            backtest: BacktestConfig::default(),
            backtest_strategy: BacktestStrategyParams::default(),
            saved_filters: Vec::new(),
            dashboards: Vec::new(),
            active_dashboard: None,
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

/// Rewrite persisted ticker fields to §67 canonical form (Issue #204 / §73).
pub fn canonicalize_persisted_symbols(cfg: &mut Config) -> SymbolCanonicalizeReport {
    canonicalize_persisted_symbol_fields(
        &mut cfg.watchlist,
        &mut cfg.portfolio,
        &mut cfg.alerts,
        &mut cfg.default_symbol,
        &mut cfg.last_symbol,
        &mut cfg.dashboards,
    )
}

/// Read `path` as JSON [`Config`]. Missing file → [`Config::default`]; same rules as [`Config::try_load`] after path resolution.
fn load_config_from_path(path: &Path) -> Result<Config, ConfigError> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(ConfigError::Io(e)),
        Ok(s) => {
            let mut cfg: Config = serde_json::from_str(&s).map_err(ConfigError::Serde)?;
            sanitize_saved_filters(&mut cfg.saved_filters);
            normalize_dashboards(&mut cfg.dashboards);
            let report = canonicalize_persisted_symbols(&mut cfg);
            if report.any_changes() {
                tracing::info!(
                    ?report,
                    "canonicalized persisted symbols while loading config"
                );
            }
            Ok(cfg)
        }
    }
}

impl Config {
    /// API key resolution (used for **Polygon** only; Issue #28 / SPEC §42.2):
    ///
    /// 1. Non-empty [`api_key`](Self::api_key) from config file (`~/.stockterm.json`).
    /// 2. Else non-empty `STOCKTERM_API_KEY` environment variable (runtime overlay only).
    /// 3. Else empty string (Polygon calls fail until configured).
    ///
    /// Does not mutate `self.api_key`. [`try_load`](Self::try_load) does not copy env into the struct.
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

    /// Load config from `~/.stockterm.json` (or defaults when missing).
    ///
    /// Does **not** copy `STOCKTERM_API_KEY` into [`api_key`](Self::api_key); use
    /// [`effective_api_key`](Self::effective_api_key) at request time (SPEC §42.2).
    pub fn try_load() -> Result<Self, ConfigError> {
        let path = match Self::config_file_path() {
            Ok(p) => p,
            Err(ConfigError::NoHomeDir) => return Ok(Config::default()),
            Err(e) => return Err(e),
        };
        load_config_from_path(&path)
    }

    /// Persist config to `~/.stockterm.json`.
    ///
    /// **Deprecated** — prefer [`try_save`](Self::try_save). On failure, logs via
    /// `tracing::error!` and does not surface in the TUI (Issue #192 / SPEC §66).
    #[deprecated(
        since = "0.1.0",
        note = "use Config::try_save; errors are logged via tracing"
    )]
    pub fn save(&self) {
        if let Err(e) = self.try_save() {
            tracing::error!(error = %e, "Config::save failed");
        }
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
    use std::sync::Mutex;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var(key).ok();
            // SAFETY: held under ENV_TEST_LOCK for the test duration.
            unsafe { std::env::set_var(key, value) };
            Self { key, prev }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => unsafe { std::env::set_var(self.key, v) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn effective_api_key_prefers_config_file_value() {
        let c = Config {
            api_key: "from-config".to_string(),
            ..Default::default()
        };
        assert_eq!(c.effective_api_key().as_ref(), "from-config");
    }

    #[test]
    fn effective_api_key_reads_env_without_mutating_config() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _guard = EnvVarGuard::set("STOCKTERM_API_KEY", "from-env");
        let c = Config::default();
        assert_eq!(c.effective_api_key().as_ref(), "from-env");
        assert!(c.api_key.is_empty());
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
        assert!(c.last_time_range.is_none());
        assert!(c.last_chart_mode.is_none());
    }

    #[test]
    fn serde_last_time_range_last_chart_mode_roundtrip() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo","last_time_range":"d1","last_chart_mode":"candles"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert_eq!(c.last_time_range.as_deref(), Some("d1"));
        assert_eq!(c.last_chart_mode.as_deref(), Some("candles"));
    }

    #[test]
    fn serde_unknown_chart_session_strings_still_load() {
        let j = r#"{"portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo","last_time_range":"bogus","last_chart_mode":"invalid"}"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert_eq!(c.last_time_range.as_deref(), Some("bogus"));
        assert_eq!(c.last_chart_mode.as_deref(), Some("invalid"));
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

    /// Issue #192 / SPEC §66 — `try_save` propagates I/O errors (Unix read-only file).
    #[cfg(unix)]
    #[test]
    fn try_save_permission_denied_returns_io_err() {
        use std::os::unix::fs::PermissionsExt;

        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/_stockterm_readonly_cfg_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join(".stockterm.json");
        fs::write(&path, "{}").expect("write config");
        let mut perms = fs::metadata(&path).expect("meta").permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&path, perms).expect("chmod ro");
        let _home = EnvVarGuard::set("HOME", dir.to_str().expect("utf8 home"));
        let res = Config::default().try_save();
        let _ = fs::remove_dir_all(&dir);
        assert!(
            matches!(res, Err(ConfigError::Io(_))),
            "expected Io error on read-only config, got {res:?}"
        );
    }

    /// Issue #192 / SPEC §66 — deprecated `save` does not panic when `try_save` fails.
    #[cfg(unix)]
    #[test]
    #[allow(deprecated)]
    fn config_save_does_not_panic_on_try_save_failure() {
        use std::os::unix::fs::PermissionsExt;

        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/_stockterm_readonly_cfg_save_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join(".stockterm.json");
        fs::write(&path, "{}").expect("write config");
        let mut perms = fs::metadata(&path).expect("meta").permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&path, perms).expect("chmod ro");
        let _home = EnvVarGuard::set("HOME", dir.to_str().expect("utf8 home"));
        let cfg = Config::default();
        assert!(matches!(cfg.try_save(), Err(ConfigError::Io(_))));
        cfg.save();
        let _ = fs::remove_dir_all(&dir);
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

    #[test]
    fn load_config_from_path_canonicalizes_mixed_case_symbols() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/_stockterm_canonicalize_cfg_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("mixed.json");
        let j = r#"{
            "portfolio":[
                {"symbol":"aapl","shares":1.0,"purchase_price":100.0},
                {"symbol":"AAPL","shares":2.0,"purchase_price":110.0}
            ],
            "watchlist":["aapl","AAPL","MSFT"],
            "refresh_rate":0,
            "api_key":"",
            "alerts":[{"symbol":"msft","condition":"Above","price":1.0,"triggered":false}],
            "default_symbol":"btc - usd",
            "last_symbol":"aapl",
            "provider":"yahoo"
        }"#;
        fs::write(&path, j).expect("write config");
        let cfg = super::load_config_from_path(&path).expect("load");
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(cfg.watchlist, vec!["AAPL", "MSFT"]);
        assert_eq!(cfg.portfolio.len(), 1);
        assert_eq!(cfg.portfolio[0].symbol, "AAPL");
        assert_eq!(cfg.alerts[0].symbol, "MSFT");
        assert_eq!(cfg.default_symbol, "BTC-USD");
        assert_eq!(cfg.last_symbol.as_deref(), Some("AAPL"));
    }

    #[test]
    fn serde_dashboards_and_active_dashboard_roundtrip() {
        let j = r#"{
            "portfolio":[],"watchlist":[],"refresh_rate":0,"api_key":"","alerts":[],"default_symbol":"","provider":"yahoo",
            "active_dashboard":"dual_watchlist",
            "dashboards":[{
                "name":"dual_watchlist","rows":1,"cols":2,
                "panes":[
                    {"id":"wl_left","kind":"watchlist","row":0,"col":0,"row_span":1,"col_span":1},
                    {"id":"wl_right","kind":"watchlist","row":0,"col":1,"row_span":1,"col_span":1}
                ]
            }]
        }"#;
        let c: Config = serde_json::from_str(j).expect("parse");
        assert_eq!(c.active_dashboard.as_deref(), Some("dual_watchlist"));
        assert_eq!(c.dashboards.len(), 1);
        assert_eq!(c.dashboards[0].panes.len(), 2);
    }
}
