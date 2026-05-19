use crate::api::error::ProviderError;
use crate::api::http::maybe_debug_http_delay;
use crate::api::market_provider_for;
use crate::api::HistoricalQuery;
use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
use crate::app::app_error::{
    push_error_log, persistence_for_app_error, ActiveErrorState, AppError, ErrorLogEntry,
    ErrorPersistence, ErrorSourceDomain, LastFailedFetch, ERROR_TRANSIENT_TTL,
};
use crate::app::charts::{viewport_zoom_in, viewport_zoom_out, ChartDisplayMode, ChartViewport};
use crate::app::event::{spawn_event_thread, Event};
use crate::app::handlers::handle_event;
use crate::app::ui::draw;
use crate::config::theme::{PaletteRgb, Theme, ThemePreset};
use crate::config::keymap::{Action, BindingLayer};
use crate::config::{
    Config, ConfigError, LayoutPreset, MarketProviderKind, ResolvedKeymap, ResolvedLayout,
};
use crate::models::alerts::{Alert, AlertCondition};
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::portfolio::PortfolioItem;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;
use crate::models::time_range::TimeRange;
use ratatui::backend::Backend;
use ratatui::widgets::{ListState, TableState};
use ratatui::Terminal;
use futures_util::future::FutureExt;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{error::SendError, UnboundedSender};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    StockView,
    Portfolio,
    Alerts,
    Search,
    News,
    Charts,
    Settings,
}

impl Tab {
    /// Stable ids in `~/.stockterm.json` (`last_tab`) — Issue #19 / §22.
    pub(crate) fn as_config_str(self) -> &'static str {
        match self {
            Tab::StockView => "stock_view",
            Tab::Portfolio => "portfolio",
            Tab::Alerts => "alerts",
            Tab::Search => "search",
            Tab::News => "news",
            Tab::Charts => "charts",
            Tab::Settings => "settings",
        }
    }

    pub(crate) fn from_config_str(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "stock_view" | "StockView" => Tab::StockView,
            "portfolio" | "Portfolio" => Tab::Portfolio,
            "alerts" | "Alerts" => Tab::Alerts,
            "search" | "Search" => Tab::Search,
            "news" | "News" => Tab::News,
            "charts" | "Charts" => Tab::Charts,
            "settings" | "Settings" => Tab::Settings,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsEdit {
    RefreshRate,
    DefaultSymbol,
}

/// Add-holding dialog field focus (Issue #6 / SPEC §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioAddField {
    Shares,
    Price,
}

/// In-modal state for adding a portfolio row (Issue #6 / SPEC §13).
#[derive(Debug, Clone)]
pub struct PortfolioAddDialog {
    pub shares_buffer: String,
    pub price_buffer: String,
    pub focused: PortfolioAddField,
    pub inline_error: Option<String>,
}

impl Default for PortfolioAddDialog {
    fn default() -> Self {
        Self {
            shares_buffer: String::new(),
            price_buffer: String::new(),
            focused: PortfolioAddField::Shares,
            inline_error: None,
        }
    }
}

/// Add-alert modal field focus (SPEC §18.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertAddField {
    Symbol,
    Condition,
    Threshold,
}

#[derive(Debug, Clone)]
pub struct AlertAddDialog {
    pub symbol_buffer: String,
    pub condition: AlertCondition,
    pub threshold_buffer: String,
    pub focused: AlertAddField,
    pub inline_error: Option<String>,
}

impl AlertAddDialog {
    pub fn new_from_app(app: &App) -> Self {
        Self {
            symbol_buffer: normalize_symbol(&app.symbol).unwrap_or_default(),
            condition: AlertCondition::Above,
            threshold_buffer: String::new(),
            focused: AlertAddField::Symbol,
            inline_error: None,
        }
    }
}

/// Clears a stuck `*_inflight` flag when `FetchDone` could not be sent (Issue #71 / SPEC §11.12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InflightRecovery {
    Historical,
    News,
    Search,
    Stock,
    /// News tab URL open/copy when [`UrlOpDone`] could not be delivered (§27).
    NewsUrlOp,
}

/// Outcomes from background HTTP tasks (never awaited on the draw/input hot path).
pub enum FetchDone {
    Stock {
        generation: u64,
        quotes: HashMap<String, TickerResponse>,
        errors: Vec<(String, ProviderError)>,
    },
    Historical {
        symbol: String,
        time_range: TimeRange,
        result: Result<HistoricalResponse, ProviderError>,
    },
    News {
        symbol: String,
        result: Result<NewsResponse, ProviderError>,
    },
    Search {
        generation: u64,
        query: String,
        result: Result<SymbolSearchResponse, ProviderError>,
    },
}

fn warn_fetch_channel_closed(context: &str, err: &SendError<FetchDone>) {
    eprintln!("stockterm: dropped {context} (channel closed): {err}");
}

fn warn_inflight_recovery_send_failed(kind: &str, err: SendError<InflightRecovery>) {
    eprintln!("stockterm: failed inflight recovery send ({kind}): {err}");
}

/// OS URL open / clipboard work completed off the input hot path (Issues #58, #59 / SPEC §27).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlOpKind {
    Open,
    Copy,
}

pub(crate) struct UrlOpDone {
    pub result: Result<(), String>,
    pub flash: Option<crate::app::open_url::NewsUrlFlashHint>,
}

/// Releases [`App::news_url_op_inflight`] when a URL task panics or cannot deliver [`UrlOpDone`].
struct NewsUrlOpInflightGuard {
    recovery_tx: Option<UnboundedSender<InflightRecovery>>,
    disarmed: bool,
}

impl NewsUrlOpInflightGuard {
    fn new(recovery_tx: Option<UnboundedSender<InflightRecovery>>) -> Self {
        Self {
            recovery_tx,
            disarmed: false,
        }
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for NewsUrlOpInflightGuard {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }
        if let Some(tx) = &self.recovery_tx {
            if let Err(e) = tx.send(InflightRecovery::NewsUrlOp) {
                eprintln!("stockterm: failed news url op inflight recovery send: {e}");
            }
        }
    }
}

pub struct App {
    pub config: Config,
    pub ticker_data: Option<TickerResponse>,
    pub historical_data: Option<HistoricalResponse>,
    pub search_results: Option<SymbolSearchResponse>,
    pub news_data: Option<NewsResponse>,
    pub should_quit: bool,
    pub should_fetch_ticker: bool,
    pub symbol: String,
    pub watchlist: Vec<String>,
    pub watchlist_quotes: HashMap<String, TickerResponse>,
    pub watchlist_state: TableState,
    pub portfolio: Vec<PortfolioItem>,
    pub portfolio_state: TableState,
    pub alerts: Vec<Alert>,
    pub alerts_state: TableState,
    pub active_tab: Tab,
    /// Issue #20 / §20 — surfaced runtime error (status bar); `error_message()` exposes line text.
    pub(crate) active_runtime_error: Option<ActiveErrorState>,
    /// Config load failure at startup (banner); distinct from runtime errors (§20.7).
    pub startup_error: Option<AppError>,
    /// Issue #13 / SPEC §24 — resolved chord→action tables per [`BindingLayer`](crate::config::keymap::BindingLayer).
    pub resolved_keymap: ResolvedKeymap,
    pub error_log: VecDeque<ErrorLogEntry>,
    pub error_log_overlay_open: bool,
    pub error_log_scroll: usize,
    /// Issue #120 / SPEC §20.15.1 — last layout-derived row count of the error
    /// log overlay's list area (excludes border + footer). Updated by
    /// `draw_error_log_overlay` every frame the overlay is open; consumed by
    /// `handle_error_log_overlay_keys` in [`crate::app::handlers`] to clamp
    /// `error_log_scroll` against the *painted* viewport. Defaults to `1` (a
    /// safe non-zero floor) until the first frame is drawn at the current
    /// terminal size.
    pub error_log_visible_rows: usize,
    pub last_failed_fetch: LastFailedFetch,
    pub search_query: String,
    pub search_table_state: TableState,
    pub search_request_generation: u64,
    pub search_refresh_inflight: bool,
    search_debounce_deadline: Option<Instant>,
    /// Issue #129 — flush session fields to disk after this instant (set by `persist_session_to_disk`).
    session_persist_deadline: Option<Instant>,
    pub news_list_state: ListState,
    /// Settings tab: selected menu row (0..SETTINGS_ROW_COUNT).
    pub settings_row: usize,
    pub settings_editing: Option<SettingsEdit>,
    pub settings_edit_buffer: String,
    pub settings_inline_error: Option<String>,
    pub settings_saved_flash_until: Option<Instant>,
    /// Throttle tick-driven network calls so quote refreshes respect `refresh_rate`.
    last_stock_network_poll: Option<Instant>,
    last_charts_network_poll: Option<Instant>,
    last_news_network_poll: Option<Instant>,
    /// True while a watchlist / quote batch is in flight.
    pub stock_refresh_inflight: bool,
    fetch_done_tx: Option<UnboundedSender<FetchDone>>,
    inflight_recovery_tx: Option<UnboundedSender<InflightRecovery>>,
    stock_fetch_generation: u64,
    stock_refresh_pending: bool,
    hist_refresh_inflight: bool,
    pub news_refresh_inflight: bool,
    url_op_tx: Option<UnboundedSender<UrlOpDone>>,
    news_url_op_inflight: bool,
    news_url_flash: Option<(crate::app::open_url::NewsUrlFlashHint, Instant)>,
    /// Charts tab: selected window (Issue #9).
    pub time_range: TimeRange,
    /// Charts tab: pan/zoom indices into `historical_data.results` (Issue #8).
    pub chart_viewport: ChartViewport,
    /// Line vs candlestick rendering (Issue #7).
    pub chart_mode: ChartDisplayMode,
    /// Issue #6 — add holding (shares / price) modal.
    pub portfolio_dialog: Option<PortfolioAddDialog>,
    /// Issue #6 — first `d` arms; second `d` or `y` confirms remove.
    pub portfolio_remove_armed: bool,
    /// SPEC §18.4 — add price alert dialog.
    pub alert_add_dialog: Option<AlertAddDialog>,
    /// SPEC §18.14.2 — `try_save` failed in `save_alerts`; retry once per stock batch.
    pub alerts_save_retry_pending: bool,
    /// Issue #14 / SPEC §21.5 — Settings Theme row: preset ring before **Enter** saves.
    pub settings_theme_draft: ThemePreset,
    /// Issue #15 / SPEC §31.7 — Settings Layout row: preset ring before **Enter** saves.
    pub settings_layout_draft: LayoutPreset,
    /// Issue #16 / SPEC §23 — substring filter (Portfolio + Stock View); cleared on tab switch.
    pub filter_query: String,
    /// Issue #16 — true after `/` until Enter (commit) or Esc (clear).
    pub filter_input_mode: bool,
}

const MISSING_API_KEY_FOR_POLYGON_MSG: &str = "Polygon provider requires a non-empty `api_key` in ~/.stockterm.json or export STOCKTERM_API_KEY.";

const MAX_CONCURRENT_QUOTES: usize = 2;

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(250);

/// Issue #129 / SPEC §22.7.4 — coalesce rapid `last_tab` / `last_symbol` disk writes.
const SESSION_PERSIST_DEBOUNCE: Duration = Duration::from_millis(400);

/// Rows in the Settings tab (refresh, default symbol, notifications, theme, provider, keymap, layout).
pub const SETTINGS_ROW_COUNT: usize = 7;

const SETTINGS_SAVED_FLASH: Duration = Duration::from_secs(2);

const NEWS_URL_FLASH: Duration = Duration::from_secs(2);

/// Trim and uppercase ticker input; returns `None` if empty after trim.
pub fn normalize_symbol(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    Some(t.to_uppercase())
}

fn quote_error_digest_for_merge(err: &AppError) -> String {
    match err {
        AppError::Provider(pe) => pe.to_string(),
        AppError::Internal(s) | AppError::ConfigSave(s) => s.clone(),
    }
}

/// Alerts-disk failure prefix for §22.2 merge; strip a prior quote tail after ` · ` before re-merge.
fn alerts_disk_failure_head_for_quote_merge(full: &str) -> &str {
    if !full.starts_with(ALERTS_SAVE_ERROR_PREFIX) {
        return full;
    }
    full.split_once(" · ").map(|(head, _)| head).unwrap_or(full)
}

async fn run_stock_quote_batch(
    generation: u64,
    symbols: Vec<String>,
    config: Config,
) -> FetchDone {
    maybe_debug_http_delay().await;

    if config.provider == MarketProviderKind::Yahoo {
        let (quotes_raw, mut errors) =
            crate::api::yahoo::yahoo_latest_quotes_for_symbols(&symbols, MAX_CONCURRENT_QUOTES).await;
        let mut quotes = HashMap::new();
        for (sym, mut data) in quotes_raw {
            if let Some(msg) = data.api_error_message() {
                errors.push((sym.clone(), ProviderError::ApiMessage(msg)));
                continue;
            }
            if data.ticker.is_empty() {
                data.ticker = sym.clone();
            }
            quotes.insert(sym, data);
        }
        return FetchDone::Stock {
            generation,
            quotes,
            errors,
        };
    }

    let provider = market_provider_for(config.provider);
    let sem = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT_QUOTES));
    let mut set = JoinSet::new();
    for sym in symbols {
        let sem = sem.clone();
        let cfg = config.clone();
        let provider = provider.clone();
        set.spawn(async move {
            let _permit = sem.acquire().await.ok();
            let res = provider.get_quote(&sym, &cfg).await;
            (sym, res)
        });
    }

    let mut quotes = HashMap::new();
    let mut errors = Vec::new();

    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((sym, Ok(mut data))) => {
                if let Some(msg) = data.api_error_message() {
                    errors.push((sym, ProviderError::ApiMessage(msg)));
                    continue;
                }
                if data.ticker.is_empty() {
                    data.ticker = sym.clone();
                }
                quotes.insert(sym, data);
            }
            Ok((sym, Err(e))) => errors.push((sym, e)),
            Err(e) => errors.push((
                String::new(),
                ProviderError::Transport(format!("task join: {e}")),
            )),
        }
    }

    FetchDone::Stock {
        generation,
        quotes,
        errors,
    }
}

impl App {
    pub fn new() -> App {
        let (config, mut startup_error) = match Config::try_load() {
            Ok(c) => (c, None),
            Err(e) => (
                Config::default(),
                Some(AppError::ConfigSave(format!(
                    "Config load failed: {e}"
                ))),
            ),
        };

        let (resolved_keymap, keymap_err) = ResolvedKeymap::build(config.keymap.as_ref());
        if let Some(ke) = keymap_err {
            startup_error = Some(match startup_error {
                Some(se) => AppError::Internal(format!("{}\n{}", se.status_line(), ke)),
                None => AppError::Internal(ke),
            });
        }
        let portfolio = config.portfolio.clone();
        let alerts = config.alerts.clone();

        let watchlist: Vec<String> = config
            .watchlist
            .iter()
            .filter_map(|s| normalize_symbol(s))
            .collect();
        let mut seen = HashSet::new();
        let watchlist: Vec<String> = watchlist
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect();

        let symbol = if let Some(first) = watchlist.first() {
            first.clone()
        } else {
            config
                .last_symbol
                .as_deref()
                .and_then(normalize_symbol)
                .or_else(|| normalize_symbol(&config.default_symbol))
                .unwrap_or_else(|| "AAPL".to_string())
        };

        let active_tab = config
            .last_tab
            .as_deref()
            .and_then(Tab::from_config_str)
            .unwrap_or(Tab::StockView);

        let mut watchlist_state = TableState::default();
        if !watchlist.is_empty() {
            watchlist_state.select(Some(0));
        }

        let settings_theme_draft = config
            .theme
            .as_ref()
            .map(Theme::effective_preset)
            .unwrap_or(ThemePreset::BuiltinDefault);

        let settings_layout_draft = config.layout.effective_preset();

        let mut app = App {
            config: config.clone(),
            ticker_data: None,
            historical_data: None,
            search_results: None,
            news_data: None,
            should_quit: false,
            should_fetch_ticker: false,
            symbol,
            watchlist,
            watchlist_quotes: HashMap::new(),
            watchlist_state,
            portfolio,
            portfolio_state: TableState::default(),
            alerts,
            alerts_state: TableState::default(),
            active_tab,
            active_runtime_error: None,
            startup_error,
            resolved_keymap,
            error_log: VecDeque::new(),
            error_log_overlay_open: false,
            error_log_scroll: 0,
            error_log_visible_rows: 1,
            last_failed_fetch: LastFailedFetch::None,
            search_query: String::new(),
            search_table_state: TableState::default(),
            search_request_generation: 0,
            search_refresh_inflight: false,
            search_debounce_deadline: None,
            session_persist_deadline: None,
            news_list_state: ListState::default(),
            settings_row: 0,
            settings_editing: None,
            settings_edit_buffer: String::new(),
            settings_inline_error: None,
            settings_saved_flash_until: None,
            last_stock_network_poll: None,
            last_charts_network_poll: None,
            last_news_network_poll: None,
            stock_refresh_inflight: false,
            fetch_done_tx: None,
            inflight_recovery_tx: None,
            stock_fetch_generation: 0,
            stock_refresh_pending: false,
            hist_refresh_inflight: false,
            news_refresh_inflight: false,
            url_op_tx: None,
            news_url_op_inflight: false,
            news_url_flash: None,
            time_range: TimeRange::default(),
            chart_viewport: ChartViewport::default(),
            chart_mode: ChartDisplayMode::default(),
            portfolio_dialog: None,
            portfolio_remove_armed: false,
            alert_add_dialog: None,
            alerts_save_retry_pending: false,
            settings_theme_draft,
            settings_layout_draft,
            filter_query: String::new(),
            filter_input_mode: false,
        };

        if !app.portfolio.is_empty() {
            app.portfolio_state.select(Some(0));
        }
        if !app.alerts.is_empty() {
            app.alerts_state.select(Some(0));
        }

        app
    }

    /// Status-line text for active runtime error (Issue #103 checks this string).
    pub fn error_message(&self) -> Option<String> {
        self.active_runtime_error
            .as_ref()
            .map(|a| a.display_line())
    }

    /// Issue #14 — palette for this frame (Settings Theme row previews `settings_theme_draft`).
    pub fn theme_palette_for_render(&self) -> PaletteRgb {
        let mut t = self.config.theme.clone().unwrap_or_default();
        if self.active_tab == Tab::Settings
            && self.settings_row == 3
            && self.settings_editing.is_none()
        {
            t.preset = Some(self.settings_theme_draft);
        }
        t.resolve_rgb()
    }

    /// Issue #15 — layout for this frame (Settings Layout row previews `settings_layout_draft`).
    ///
    /// Mirrors [`Self::theme_palette_for_render`]: clone saved layout, swap only `preset` while
    /// focused on row 6 so scalar overrides (e.g. hand-tuned `charts_chart_pct`) stay visible.
    pub fn layout_for_render(&self) -> ResolvedLayout {
        let mut layout = self.config.layout.clone();
        if self.active_tab == Tab::Settings
            && self.settings_row == 6
            && self.settings_editing.is_none()
        {
            layout.preset = Some(self.settings_layout_draft);
        }
        layout.resolve()
    }

    pub(crate) fn sync_settings_theme_draft_from_config(&mut self) {
        self.settings_theme_draft = self
            .config
            .theme
            .as_ref()
            .map(Theme::effective_preset)
            .unwrap_or(ThemePreset::BuiltinDefault);
    }

    pub(crate) fn surface_runtime_error(
        &mut self,
        tab: Tab,
        domain: ErrorSourceDomain,
        err: AppError,
        push_one_log_line: bool,
    ) {
        if push_one_log_line {
            push_error_log(
                &mut self.error_log,
                tab,
                err.category(),
                err.status_line(),
            );
            // SPEC §20.15.2 — ring eviction shrinks the log, so re-clamp the
            // overlay scroll. Without this, a user scrolled to the bottom of
            // the overlay sees a "dead `k`" once after each new push (Issue
            // #120 / #121 audit follow-up).
            self.clamp_error_log_scroll();
        }
        let persistence = persistence_for_app_error(&err);
        self.active_runtime_error = Some(ActiveErrorState::new(
            err,
            persistence,
            Instant::now(),
            domain,
        ));
    }

    /// Copy `active_tab` / `symbol` into `config` before any disk write (Issue #19 / §22).
    pub(crate) fn sync_session_fields_into_config(&mut self) {
        self.config.last_tab = Some(self.active_tab.as_config_str().to_string());
        self.config.last_symbol = normalize_symbol(&self.symbol);
    }

    /// [`Config::try_save`] after refreshing `last_tab` / `last_symbol` from UI state.
    pub(crate) fn try_save_config_with_session(&mut self) -> Result<(), ConfigError> {
        self.sync_session_fields_into_config();
        self.config.try_save()
    }

    /// Schedule a debounced flush of `last_tab` / `last_symbol` to `~/.stockterm.json` (Issue #129).
    /// High-frequency callers (`j`/`k`, tab keys, Stock View **Enter** after fetch) coalesce into one write.
    fn persist_session_to_disk(&mut self) {
        self.session_persist_deadline = Some(Instant::now() + SESSION_PERSIST_DEBOUNCE);
    }

    fn flush_session_persist_if_due(&mut self) {
        let Some(deadline) = self.session_persist_deadline else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        self.session_persist_deadline = None;
        if let Err(e) = self.try_save_config_with_session() {
            self.surface_runtime_error(
                self.active_tab,
                ErrorSourceDomain::Other,
                AppError::ConfigSave(format!("Failed to save session: {e}")),
                false,
            );
        }
    }

    /// Issue #120 / #121 / SPEC §20.15.1 — clamp [`Self::error_log_scroll`]
    /// against the most recently rendered visible-row count and the current
    /// [`Self::error_log`] length. Idempotent; safe to call from input
    /// handlers and on overlay open/toggle. Render code in
    /// [`crate::app::ui`] must NOT mutate `error_log_scroll`; it only
    /// publishes [`Self::error_log_visible_rows`] each frame.
    pub(crate) fn clamp_error_log_scroll(&mut self) {
        let total = self.error_log.len();
        let visible = self.error_log_visible_rows.max(1);
        let max_scroll = total.saturating_sub(visible);
        if self.error_log_scroll > max_scroll {
            self.error_log_scroll = max_scroll;
        }
    }

    /// Active runtime error is an alerts `try_save` failure or a §22.2 merged line (`Internal` + same prefix).
    fn active_alerts_save_failure_message(&self) -> Option<&str> {
        self.active_runtime_error.as_ref().and_then(|a| match &a.error {
            AppError::ConfigSave(s) if s.starts_with(ALERTS_SAVE_ERROR_PREFIX) => Some(s.as_str()),
            AppError::Internal(s) if s.starts_with(ALERTS_SAVE_ERROR_PREFIX) => Some(s.as_str()),
            _ => None,
        })
    }

    /// True when the status line shows an alerts-disk failure (including §22.2 merged `Internal`).
    fn preserves_alerts_save_banner(&self) -> bool {
        self.active_alerts_save_failure_message().is_some()
    }

    pub(crate) fn clear_alerts_save_runtime_error_after_recovery(&mut self) {
        if self.preserves_alerts_save_banner() {
            self.active_runtime_error = None;
        }
    }

    pub(crate) fn clear_active_runtime_unless_alerts_save(&mut self) {
        if !self.preserves_alerts_save_banner() {
            self.active_runtime_error = None;
        }
    }

    fn tick_runtime_error_ttl(&mut self) {
        let Some(active) = &self.active_runtime_error else {
            return;
        };
        if active.persistence != ErrorPersistence::Transient {
            return;
        }
        if active.shown_since.elapsed() >= ERROR_TRANSIENT_TTL {
            self.active_runtime_error = None;
        }
    }

    /// §20.5 — `Ctrl+R` user retry (bypasses throttle once per domain).
    pub fn retry_last_failed_fetch(&mut self) {
        match &self.last_failed_fetch {
            LastFailedFetch::StockQuoteBatch => {
                self.last_stock_network_poll = None;
                self.request_immediate_stock_poll();
            }
            LastFailedFetch::Historical => {
                self.last_charts_network_poll = None;
                if !self.hist_refresh_inflight {
                    self.try_spawn_historical_fetch();
                }
            }
            LastFailedFetch::News { .. } => {
                self.last_news_network_poll = None;
                if !self.news_refresh_inflight {
                    self.try_spawn_news_fetch();
                }
            }
            LastFailedFetch::Search { .. } => {
                if !self.search_refresh_inflight && !self.search_query.trim().is_empty() {
                    self.search_debounce_deadline = Some(Instant::now());
                    self.try_spawn_search_tick();
                }
            }
            LastFailedFetch::None => {}
        }
    }

    fn provider_ready(&self) -> bool {
        match self.config.provider {
            MarketProviderKind::Yahoo => true,
            MarketProviderKind::Polygon => !self.config.effective_api_key().is_empty(),
        }
    }

    fn data_poll_interval(&self) -> Duration {
        Duration::from_secs(data_poll_interval_secs(self.config.refresh_rate))
    }

    /// Clears throttle timestamps so the next tick may poll immediately (Issue #4 / SPEC §35.6.2).
    fn reset_network_poll_clocks(&mut self) {
        self.last_stock_network_poll = None;
        self.last_charts_network_poll = None;
        self.last_news_network_poll = None;
    }

    pub(crate) fn collect_symbols_for_quote_fetch(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for s in &self.watchlist {
            if seen.insert(s.clone()) {
                out.push(s.clone());
            }
        }
        if let Some(sym) = normalize_symbol(&self.symbol) {
            if seen.insert(sym.clone()) {
                out.push(sym);
            }
        }
        for item in &self.portfolio {
            if let Some(sym) = normalize_symbol(&item.symbol) {
                if seen.insert(sym.clone()) {
                    out.push(sym);
                }
            }
        }
        out
    }

    fn clear_portfolio_tab_transient(&mut self) {
        self.portfolio_dialog = None;
        self.portfolio_remove_armed = false;
    }

    /// Issue #16 / SPEC §23 — reset filter on any tab change.
    fn clear_table_filter(&mut self) {
        self.filter_query.clear();
        self.filter_input_mode = false;
    }

    pub(crate) fn watchlist_filter_indices(&self) -> Vec<usize> {
        crate::app::table_filter::filter_symbol_indices(&self.watchlist, &self.filter_query)
    }

    pub(crate) fn portfolio_filter_indices(&self) -> Vec<usize> {
        crate::app::table_filter::filter_row_indices(self.portfolio.len(), |i| {
            self.portfolio[i].symbol.as_str()
        }, &self.filter_query)
    }

    pub(crate) fn clamp_portfolio_filter_selection(&mut self) {
        let f = self.portfolio_filter_indices();
        if f.is_empty() {
            self.portfolio_state.select(None);
            return;
        }
        let max_i = f.len() - 1;
        let new_sel = self
            .portfolio_state
            .selected()
            .map(|s| s.min(max_i))
            .unwrap_or(0);
        self.portfolio_state.select(Some(new_sel));
    }

    pub(crate) fn clamp_watchlist_filter_selection(&mut self) {
        let f = self.watchlist_filter_indices();
        if f.is_empty() {
            self.watchlist_state.select(None);
            return;
        }
        let max_i = f.len() - 1;
        let new_sel = self
            .watchlist_state
            .selected()
            .map(|s| s.min(max_i))
            .unwrap_or(0);
        self.watchlist_state.select(Some(new_sel));
    }

    fn clamp_both_filter_selections(&mut self) {
        self.clamp_portfolio_filter_selection();
        self.clamp_watchlist_filter_selection();
    }

    /// Issue #16 / #137 — while `filter_input_mode`, resolves [`BindingLayer::FilterInput`]
    /// (SPEC §28); returns true when mode is active (swallows unmapped keys).
    pub(crate) fn consume_filter_input_key(&mut self, key: &crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        use Action::*;

        if !self.filter_input_mode {
            return false;
        }

        let Some(action) = self
            .resolved_keymap
            .action(BindingLayer::FilterInput, key)
        else {
            return true;
        };

        if key.modifiers != KeyModifiers::NONE {
            return true;
        }

        match action {
            FilterClear => {
                self.filter_query.clear();
                self.filter_input_mode = false;
            }
            FilterCommit => {
                self.filter_input_mode = false;
            }
            FilterBackspace => {
                self.filter_query.pop();
            }
            FilterSlash if self.filter_query.is_empty() => {
                self.filter_input_mode = false;
            }
            FilterQueryChar => {
                if let KeyCode::Char(c) = key.code {
                    if c.is_ascii_alphanumeric()
                        && self.filter_query.len()
                            < crate::app::table_filter::MAX_FILTER_QUERY_LEN
                    {
                        self.filter_query.push(c);
                    }
                }
            }
            _ => {}
        }
        self.clamp_both_filter_selections();
        true
    }

    /// User-driven refresh (Enter, portfolio jump, etc.). Coalesces if a batch is already running.
    pub fn request_immediate_stock_poll(&mut self) {
        if self.stock_refresh_inflight {
            self.stock_refresh_pending = true;
            return;
        }
        self.spawn_stock_fetch_task();
    }

    fn try_spawn_stock_poll_throttled(&mut self) {
        if self.stock_refresh_inflight {
            return;
        }
        let due = self
            .last_stock_network_poll
            .map(|t| t.elapsed() >= self.data_poll_interval())
            .unwrap_or(true);
        if !due {
            return;
        }
        self.spawn_stock_fetch_task();
    }

    fn spawn_stock_fetch_task(&mut self) {
        let symbols = self.collect_symbols_for_quote_fetch();
        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };

        if symbols.is_empty() {
            return;
        }

        if !self.provider_ready() {
            self.surface_runtime_error(
                self.active_tab,
                ErrorSourceDomain::Stock,
                AppError::Internal(MISSING_API_KEY_FOR_POLYGON_MSG.to_string()),
                true,
            );
            self.ticker_data = None;
            return;
        }

        self.stock_refresh_inflight = true;
        self.stock_fetch_generation += 1;
        let generation = self.stock_fetch_generation;
        let cfg = self.config.clone();
        let recovery_tx = self.inflight_recovery_tx.clone();

        tokio::spawn(async move {
            let done = match AssertUnwindSafe(run_stock_quote_batch(generation, symbols, cfg))
                .catch_unwind()
                .await
            {
                Ok(done) => done,
                Err(_) => FetchDone::Stock {
                    generation,
                    quotes: HashMap::new(),
                    errors: vec![(
                        String::new(),
                        ProviderError::ApiMessage("quote batch task panicked".into()),
                    )],
                },
            };
            if let Err(e) = tx.send(done) {
                warn_fetch_channel_closed("stock quote batch result", &e);
                if let Some(rtx) = recovery_tx {
                    if let Err(re) = rtx.send(InflightRecovery::Stock) {
                        warn_inflight_recovery_send_failed("stock", re);
                    }
                }
            }
        });
    }

    fn apply_stock_fetch_done(
        &mut self,
        generation: u64,
        quotes: HashMap<String, TickerResponse>,
        errors: Vec<(String, ProviderError)>,
    ) {
        // Stale batch: a newer `stock_fetch_generation` means another batch is authoritative; keep
        // `stock_refresh_inflight` unchanged (still true if a newer batch is in flight). SPEC §16.2.1.
        if generation != self.stock_fetch_generation {
            return;
        }

        self.stock_refresh_inflight = false;
        self.last_stock_network_poll = Some(Instant::now());

        for (k, v) in quotes {
            self.watchlist_quotes.insert(k, v);
        }

        self.ticker_data = self.watchlist_quotes.get(&self.symbol).cloned();

        if !errors.is_empty() {
            self.last_failed_fetch = LastFailedFetch::StockQuoteBatch;
            for (sym, pe) in &errors {
                let ae = AppError::Provider(pe.clone());
                let line = if sym.is_empty() {
                    ae.status_line()
                } else {
                    format!("{sym}: {}", pe)
                };
                push_error_log(&mut self.error_log, self.active_tab, ae.category(), line);
            }
            // SPEC §20.15.2 — ring eviction (one per push above the cap)
            // shrinks the log; re-clamp once after the batch so a user
            // scrolled to the bottom of the overlay does not observe a
            // "dead `k`" key (Issue #120 / #121 audit follow-up).
            self.clamp_error_log_scroll();
            let alerts_save_line = self
                .active_alerts_save_failure_message()
                .map(|full| alerts_disk_failure_head_for_quote_merge(full).to_string());

            let primary_base = if self.ticker_data.is_none() && errors.len() == 1 {
                AppError::Provider(errors[0].1.clone())
            } else if self.ticker_data.is_none() {
                AppError::Internal(
                    errors
                        .iter()
                        .map(|(s, e)| format!("{s}: {e}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                )
            } else {
                AppError::Internal(format!(
                    "Some quotes failed: {}",
                    errors
                        .iter()
                        .map(|(s, e)| format!("{s}: {e}"))
                        .collect::<Vec<_>>()
                        .join("; ")
                ))
            };

            let primary = if let Some(alerts_line) = alerts_save_line {
                AppError::Internal(format!(
                    "{alerts_line} · {}",
                    quote_error_digest_for_merge(&primary_base)
                ))
            } else {
                primary_base
            };

            self.surface_runtime_error(
                Tab::StockView,
                ErrorSourceDomain::Stock,
                primary,
                false,
            );
        } else if !self.preserves_alerts_save_banner() {
            self.active_runtime_error = None;
            self.last_failed_fetch = LastFailedFetch::None;
        }

        for item in &mut self.portfolio {
            if let Some(resp) = self.watchlist_quotes.get(&item.symbol) {
                if let Some(bar) = resp.latest_result() {
                    item.current_price = Some(bar.c);
                }
            }
        }

        self.check_alerts();
        self.retry_alerts_save_if_pending();

        if self.stock_refresh_pending {
            self.stock_refresh_pending = false;
            self.spawn_stock_fetch_task();
        }
    }

    fn try_spawn_historical_fetch(&mut self) {
        if self.hist_refresh_inflight {
            return;
        }
        if self.symbol.is_empty() {
            return;
        }
        let due = self
            .last_charts_network_poll
            .map(|t| t.elapsed() >= self.data_poll_interval())
            .unwrap_or(true);
        if !due {
            return;
        }

        if !self.provider_ready() {
            self.surface_runtime_error(
                Tab::Charts,
                ErrorSourceDomain::Charts,
                AppError::Internal(MISSING_API_KEY_FOR_POLYGON_MSG.to_string()),
                true,
            );
            self.historical_data = None;
            return;
        }

        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };

        self.hist_refresh_inflight = true;
        let sym = self.symbol.clone();
        let cfg = self.config.clone();
        let tr = self.time_range;
        let recovery_tx = self.inflight_recovery_tx.clone();
        tokio::spawn(async move {
            let params = tr.historical_params(chrono::Local::now());
            let from = params.from.clone();
            let to = params.to.clone();
            let hq = HistoricalQuery {
                from: &from,
                to: &to,
                bar_interval: params.bar_interval,
                yahoo_range: params.yahoo_range,
                polygon_multiplier: params.polygon_multiplier,
                polygon_timespan: params.polygon_timespan,
            };
            let provider = market_provider_for(cfg.provider);
            let result = provider.get_historical(&sym, &hq, &cfg).await;
            if let Err(e) = tx.send(FetchDone::Historical {
                symbol: sym,
                time_range: tr,
                result,
            }) {
                warn_fetch_channel_closed("historical fetch result", &e);
                if let Some(rtx) = recovery_tx {
                    if let Err(re) = rtx.send(InflightRecovery::Historical) {
                        warn_inflight_recovery_send_failed("historical", re);
                    }
                }
            }
        });
    }

    fn try_spawn_news_fetch(&mut self) {
        if self.news_refresh_inflight {
            return;
        }
        if self.symbol.is_empty() {
            return;
        }
        let due = self
            .last_news_network_poll
            .map(|t| t.elapsed() >= self.data_poll_interval())
            .unwrap_or(true);
        if !due {
            return;
        }

        if !self.provider_ready() {
            self.surface_runtime_error(
                Tab::News,
                ErrorSourceDomain::News,
                AppError::Internal(MISSING_API_KEY_FOR_POLYGON_MSG.to_string()),
                true,
            );
            self.news_data = None;
            return;
        }

        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };

        self.news_refresh_inflight = true;
        let sym = self.symbol.clone();
        let cfg = self.config.clone();
        let recovery_tx = self.inflight_recovery_tx.clone();
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.get_news(&sym, &cfg).await;
            if let Err(e) = tx.send(FetchDone::News {
                symbol: sym,
                result,
            }) {
                warn_fetch_channel_closed("news fetch result", &e);
                if let Some(rtx) = recovery_tx {
                    if let Err(re) = rtx.send(InflightRecovery::News) {
                        warn_inflight_recovery_send_failed("news", re);
                    }
                }
            }
        });
    }

    fn on_background_tick(&mut self) {
        match self.active_tab {
            Tab::StockView | Tab::Alerts => self.try_spawn_stock_poll_throttled(),
            Tab::Charts => self.try_spawn_historical_fetch(),
            Tab::News => self.try_spawn_news_fetch(),
            Tab::Search => self.try_spawn_search_tick(),
            _ => {}
        }
        self.tick_runtime_error_ttl();
        self.flush_session_persist_if_due();
    }

    /// Call after `search_query` mutates (typing); schedules debounced API call on Search tab.
    pub fn touch_search_debounce(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_debounce_deadline = None;
            return;
        }
        self.search_debounce_deadline = Some(Instant::now() + SEARCH_DEBOUNCE);
    }

    /// Clear search UI and invalidate in-flight responses (Esc).
    ///
    /// Issue #60 / SPEC §33 — clears [`Self::active_runtime_error`] only when
    /// [`ErrorSourceDomain::Search`]; cross-tab errors (e.g. Stock quotes) persist.
    pub fn search_esc_reset(&mut self) {
        self.search_query.clear();
        self.search_results = None;
        self.search_table_state.select(None);
        self.search_debounce_deadline = None;
        self.search_request_generation = self.search_request_generation.wrapping_add(1);
        if self
            .active_runtime_error
            .as_ref()
            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Search)
        {
            self.active_runtime_error = None;
        }
    }

    fn try_spawn_search_tick(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results = None;
            self.search_debounce_deadline = None;
            return;
        }
        if !self.provider_ready() {
            self.surface_runtime_error(
                Tab::Search,
                ErrorSourceDomain::Search,
                AppError::Internal(MISSING_API_KEY_FOR_POLYGON_MSG.to_string()),
                true,
            );
            return;
        }
        let Some(deadline) = self.search_debounce_deadline else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        if self.search_refresh_inflight {
            return;
        }
        self.search_debounce_deadline = None;
        self.spawn_search_task();
    }

    fn spawn_search_task(&mut self) {
        let query = self.search_query.clone();
        if query.trim().is_empty() || !self.provider_ready() {
            return;
        }
        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };
        self.search_request_generation = self.search_request_generation.wrapping_add(1);
        let generation = self.search_request_generation;
        let cfg = self.config.clone();
        self.search_refresh_inflight = true;
        let recovery_tx = self.inflight_recovery_tx.clone();
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.search_symbols(&query, &cfg).await;
            if let Err(e) = tx.send(FetchDone::Search {
                generation,
                query,
                result,
            }) {
                warn_fetch_channel_closed("search fetch result", &e);
                if let Some(rtx) = recovery_tx {
                    if let Err(re) = rtx.send(InflightRecovery::Search) {
                        warn_inflight_recovery_send_failed("search", re);
                    }
                }
            }
        });
    }

    /// Clear chart series when the active ticker changes (Issue #62 / SPEC §11.11.1).
    pub fn on_active_symbol_changed_for_charts(&mut self) {
        self.historical_data = None;
        self.chart_viewport = ChartViewport::default();
        self.last_charts_network_poll = None;
    }

    /// Clear news UI when the active symbol changes while on the News tab (SPEC §10.3).
    pub fn notify_symbol_changed_for_news(&mut self) {
        if self.active_tab != Tab::News {
            return;
        }
        self.news_data = None;
        self.news_list_state.select(None);
        self.last_news_network_poll = None;
    }

    pub fn search_results_len(&self) -> usize {
        self.search_results
            .as_ref()
            .map(|r| r.results.len())
            .unwrap_or(0)
    }

    pub fn search_select_next(&mut self) {
        let n = self.search_results_len();
        if n == 0 {
            return;
        }
        let i = match self.search_table_state.selected() {
            None => 0,
            Some(i) => (i + 1).min(n - 1),
        };
        self.search_table_state.select(Some(i));
    }

    pub fn search_select_prev(&mut self) {
        let n = self.search_results_len();
        if n == 0 {
            return;
        }
        let i = match self.search_table_state.selected() {
            None => 0,
            Some(i) => i.saturating_sub(1),
        };
        self.search_table_state.select(Some(i));
    }

    pub fn search_pick_symbol_go_stock(&mut self) {
        let n = self.search_results_len();
        if n == 0 {
            return;
        }
        let i = self
            .search_table_state
            .selected()
            .unwrap_or(0)
            .min(n - 1);
        let Some(row) = self
            .search_results
            .as_ref()
            .and_then(|r| r.results.get(i))
        else {
            return;
        };
        let Some(sym) = normalize_symbol(&row.ticker) else {
            return;
        };
        self.symbol = sym;
        self.on_active_symbol_changed_for_charts();
        self.notify_symbol_changed_for_news();
        self.active_tab = Tab::StockView;
        self.sync_watchlist_selection_to_symbol();
        self.request_immediate_stock_poll();
    }

    pub fn news_select_next(&mut self) {
        let n = self
            .news_data
            .as_ref()
            .map(|d| d.results.len())
            .unwrap_or(0);
        if n == 0 {
            return;
        }
        let i = match self.news_list_state.selected() {
            None => 0,
            Some(i) => (i + 1).min(n - 1),
        };
        self.news_list_state.select(Some(i));
    }

    pub fn news_select_prev(&mut self) {
        let n = self
            .news_data
            .as_ref()
            .map(|d| d.results.len())
            .unwrap_or(0);
        if n == 0 {
            return;
        }
        let i = match self.news_list_state.selected() {
            None => 0,
            Some(i) => i.saturating_sub(1),
        };
        self.news_list_state.select(Some(i));
    }

    fn news_selected_article_url(&self) -> Option<String> {
        let data = self.news_data.as_ref()?;
        let n = data.results.len();
        if n == 0 {
            return None;
        }
        let i = self.news_list_state.selected().unwrap_or(0).min(n - 1);
        data.results.get(i).map(|item| item.article_url.clone())
    }

    fn surface_news_url_validation_error(&mut self, msg: &str) {
        self.surface_runtime_error(
            Tab::News,
            ErrorSourceDomain::NewsOpenUrl,
            AppError::Internal(msg.to_string()),
            true,
        );
    }

    fn spawn_url_op(&mut self, kind: UrlOpKind, url: String) {
        if self.news_url_op_inflight {
            return;
        }
        let Some(tx) = self.url_op_tx.clone() else {
            return;
        };
        let recovery_tx = self.inflight_recovery_tx.clone();
        self.news_url_op_inflight = true;
        tokio::spawn(async move {
            let mut inflight_guard = NewsUrlOpInflightGuard::new(recovery_tx);
            let (result, flash) = match tokio::task::spawn_blocking(move || match kind {
                UrlOpKind::Open => crate::app::open_url::run_open_with_copy_fallback(&url),
                UrlOpKind::Copy => {
                    let r = crate::app::open_url::copy_article_url_blocking(&url);
                    let flash = r.as_ref().ok().map(|_| {
                        crate::app::open_url::NewsUrlFlashHint::Copied
                    });
                    (r, flash)
                }
            })
            .await
            {
                Ok(pair) => pair,
                Err(e) => (Err(e.to_string()), None),
            };
            if tx.send(UrlOpDone { result, flash }).is_ok() {
                inflight_guard.disarm();
            } else {
                eprintln!("stockterm: dropped url op result (channel closed)");
            }
        });
    }

    fn try_spawn_news_url_op(&mut self, kind: UrlOpKind) {
        let Some(raw) = self.news_selected_article_url() else {
            return;
        };
        let url = match crate::app::open_url::normalize_article_url(&raw) {
            Ok(u) => u,
            Err(msg) => {
                self.surface_news_url_validation_error(msg);
                return;
            }
        };
        self.spawn_url_op(kind, url);
    }

    pub fn news_try_open_selected(&mut self) {
        self.try_spawn_news_url_op(UrlOpKind::Open);
    }

    pub fn news_try_copy_selected(&mut self) {
        self.try_spawn_news_url_op(UrlOpKind::Copy);
    }

    fn apply_url_op_done(&mut self, msg: UrlOpDone) {
        self.news_url_op_inflight = false;
        let UrlOpDone { result, flash } = msg;
        match result {
            Ok(()) => {
                if self.active_runtime_error.as_ref().is_some_and(|a| {
                    a.source_domain == ErrorSourceDomain::NewsOpenUrl
                }) {
                    self.active_runtime_error = None;
                }
                if let Some(hint) = flash {
                    self.news_url_flash = Some((hint, Instant::now() + NEWS_URL_FLASH));
                }
            }
            Err(e) => {
                self.surface_runtime_error(
                    Tab::News,
                    ErrorSourceDomain::NewsOpenUrl,
                    AppError::Internal(e),
                    true,
                );
            }
        }
    }

    pub(crate) fn news_url_flash_line(&self) -> Option<&'static str> {
        self.news_url_flash.as_ref().and_then(|(hint, until)| {
            if Instant::now() < *until {
                Some(hint.status_text())
            } else {
                None
            }
        })
    }

    pub fn settings_row_prev(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        let prev_row = self.settings_row;
        self.settings_row = self.settings_row.saturating_sub(1);
        if prev_row != 3 && self.settings_row == 3 {
            self.sync_settings_theme_draft_from_config();
        }
        if prev_row != 6 && self.settings_row == 6 {
            self.sync_settings_layout_draft_from_config();
        }
    }

    pub fn settings_row_next(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        let prev_row = self.settings_row;
        self.settings_row = (self.settings_row + 1).min(SETTINGS_ROW_COUNT - 1);
        if prev_row != 3 && self.settings_row == 3 {
            self.sync_settings_theme_draft_from_config();
        }
        if prev_row != 6 && self.settings_row == 6 {
            self.sync_settings_layout_draft_from_config();
        }
    }

    pub fn settings_begin_edit(&mut self) {
        self.settings_inline_error = None;
        match self.settings_row {
            0 => {
                self.settings_editing = Some(SettingsEdit::RefreshRate);
                self.settings_edit_buffer = self.config.refresh_rate.to_string();
            }
            1 => {
                self.settings_editing = Some(SettingsEdit::DefaultSymbol);
                self.settings_edit_buffer = self.config.default_symbol.clone();
            }
            _ => {}
        }
    }

    pub fn settings_cancel_edit(&mut self) {
        self.settings_editing = None;
        self.settings_edit_buffer.clear();
        self.settings_inline_error = None;
    }

    /// Commit settings edit (`Enter` in edit mode). Returns `true` if the row was handled.
    pub fn settings_commit_edit(&mut self) -> bool {
        let Some(field) = self.settings_editing else {
            return false;
        };
        self.settings_inline_error = None;
        match field {
            SettingsEdit::RefreshRate => {
                let trimmed = self.settings_edit_buffer.trim();
                let Ok(v) = trimmed.parse::<u64>() else {
                    self.settings_inline_error = Some("Refresh rate must be a positive integer.".into());
                    return true;
                };
                if v < 1 {
                    self.settings_inline_error = Some("Refresh rate must be at least 1.".into());
                    return true;
                }
                self.config.refresh_rate = v;
                if let Err(e) = self.try_save_config_with_session() {
                    self.surface_runtime_error(
                        Tab::Settings,
                        ErrorSourceDomain::Settings,
                        AppError::ConfigSave(format!("Failed to save settings: {e}")),
                        true,
                    );
                } else {
                    if self.active_runtime_error.as_ref().is_some_and(|a| {
                        a.source_domain == ErrorSourceDomain::Settings
                    }) {
                        self.active_runtime_error = None;
                    }
                    self.reset_network_poll_clocks();
                    self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
                }
            }
            SettingsEdit::DefaultSymbol => {
                let Some(sym) = normalize_symbol(&self.settings_edit_buffer) else {
                    self.settings_inline_error =
                        Some("Default symbol cannot be empty.".into());
                    return true;
                };
                self.config.default_symbol = sym;
                if let Err(e) = self.try_save_config_with_session() {
                    self.surface_runtime_error(
                        Tab::Settings,
                        ErrorSourceDomain::Settings,
                        AppError::ConfigSave(format!("Failed to save settings: {e}")),
                        true,
                    );
                } else {
                    if self.active_runtime_error.as_ref().is_some_and(|a| {
                        a.source_domain == ErrorSourceDomain::Settings
                    }) {
                        self.active_runtime_error = None;
                    }
                    self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
                }
            }
        }
        self.settings_editing = None;
        self.settings_edit_buffer.clear();
        true
    }

    pub fn settings_try_enter_row(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        match self.settings_row {
            0 | 1 => self.settings_begin_edit(),
            2 => self.settings_toggle_notifications(),
            3 => self.settings_commit_theme_preset(),
            6 => self.settings_commit_layout_preset(),
            _ => {}
        }
    }

    /// Issue #14 — persist `settings_theme_draft` as the active `Config.theme` preset.
    pub fn settings_commit_theme_preset(&mut self) {
        let previous = self.config.theme.clone();
        let mut merged = previous.clone().unwrap_or_default();
        merged.preset = Some(self.settings_theme_draft);
        self.config.theme = Some(merged);
        if let Err(e) = self.try_save_config_with_session() {
            self.config.theme = previous;
            self.surface_runtime_error(
                Tab::Settings,
                ErrorSourceDomain::Settings,
                AppError::ConfigSave(format!("Failed to save theme: {e}")),
                true,
            );
        } else {
            if self.active_runtime_error.as_ref().is_some_and(|a| {
                a.source_domain == ErrorSourceDomain::Settings
            }) {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
        }
    }

    pub fn settings_cycle_theme_draft_next(&mut self) {
        self.settings_theme_draft = self.settings_theme_draft.next();
    }

    pub fn settings_cycle_theme_draft_prev(&mut self) {
        self.settings_theme_draft = self.settings_theme_draft.prev();
    }

    pub(crate) fn sync_settings_layout_draft_from_config(&mut self) {
        self.settings_layout_draft = self.config.layout.effective_preset();
    }

    /// Issue #15 — persist `settings_layout_draft` as the active `Config.layout` preset.
    ///
    /// Mirrors [`Self::settings_commit_theme_preset`]: update only `preset`, keep scalar overrides.
    pub fn settings_commit_layout_preset(&mut self) {
        let previous = self.config.layout.clone();
        let mut merged = previous.clone();
        merged.preset = Some(self.settings_layout_draft);
        self.config.layout = merged;
        if let Err(e) = self.try_save_config_with_session() {
            self.config.layout = previous;
            self.surface_runtime_error(
                Tab::Settings,
                ErrorSourceDomain::Settings,
                AppError::ConfigSave(format!("Failed to save layout: {e}")),
                true,
            );
        } else {
            if self.active_runtime_error.as_ref().is_some_and(|a| {
                a.source_domain == ErrorSourceDomain::Settings
            }) {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
        }
    }

    pub fn settings_cycle_layout_draft_next(&mut self) {
        self.settings_layout_draft = self.settings_layout_draft.next();
    }

    pub fn settings_cycle_layout_draft_prev(&mut self) {
        self.settings_layout_draft = self.settings_layout_draft.prev();
    }

    /// SPEC §18.7 — toggle desktop toasts for alert fires (bell always rings).
    pub fn settings_toggle_notifications(&mut self) {
        self.config.notifications_enabled = !self.config.notifications_enabled;
        if let Err(e) = self.try_save_config_with_session() {
            self.config.notifications_enabled = !self.config.notifications_enabled;
            self.surface_runtime_error(
                Tab::Settings,
                ErrorSourceDomain::Settings,
                AppError::ConfigSave(format!("Failed to save settings: {e}")),
                true,
            );
        } else {
            if self.active_runtime_error.as_ref().is_some_and(|a| {
                a.source_domain == ErrorSourceDomain::Settings
            }) {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
        }
    }

    fn apply_inflight_recovery(&mut self, kind: InflightRecovery) {
        match kind {
            InflightRecovery::Historical => self.hist_refresh_inflight = false,
            InflightRecovery::News => self.news_refresh_inflight = false,
            InflightRecovery::Search => self.search_refresh_inflight = false,
            InflightRecovery::Stock => {
                self.stock_refresh_inflight = false;
                // Issue #77 / SPEC §16.3: coalesced refresh must not stick pending when FetchDone send failed.
                if std::mem::take(&mut self.stock_refresh_pending) {
                    self.spawn_stock_fetch_task();
                }
            }
            InflightRecovery::NewsUrlOp => self.news_url_op_inflight = false,
        }
    }

    fn apply_fetch_done(&mut self, msg: FetchDone) {
        match msg {
            FetchDone::Stock {
                generation,
                quotes,
                errors,
            } => self.apply_stock_fetch_done(generation, quotes, errors),
            FetchDone::Historical {
                symbol,
                time_range,
                result,
            } => {
                self.hist_refresh_inflight = false;
                if symbol != self.symbol || time_range != self.time_range {
                    self.last_charts_network_poll = None;
                    return;
                }
                self.last_charts_network_poll = Some(Instant::now());
                match result {
                    Ok(data) => {
                        let prev = self.historical_data.as_ref();
                        self.chart_viewport = crate::app::charts::chart_viewport_after_refresh(
                            prev,
                            self.chart_viewport,
                            &data,
                            &symbol,
                        );
                        self.historical_data = Some(data);
                        if matches!(self.last_failed_fetch, LastFailedFetch::Historical) {
                            self.last_failed_fetch = LastFailedFetch::None;
                        }
                        if self.active_runtime_error.as_ref().is_some_and(|a| {
                            a.source_domain == ErrorSourceDomain::Charts
                        }) {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                    }
                    Err(err) => {
                        self.last_failed_fetch = LastFailedFetch::Historical;
                        self.surface_runtime_error(
                            Tab::Charts,
                            ErrorSourceDomain::Charts,
                            AppError::Provider(err),
                            true,
                        );
                        if self.historical_data.is_none() {
                            self.chart_viewport = ChartViewport::default();
                        }
                    }
                }
            }
            FetchDone::News { symbol, result } => {
                self.news_refresh_inflight = false;
                self.last_news_network_poll = Some(Instant::now());
                if symbol != self.symbol {
                    return;
                }
                match result {
                    Ok(data) => {
                        let n = data.results.len();
                        self.news_data = Some(data);
                        if matches!(self.last_failed_fetch, LastFailedFetch::News { .. }) {
                            self.last_failed_fetch = LastFailedFetch::None;
                        }
                        if self.active_runtime_error.as_ref().is_some_and(|a| {
                            a.source_domain == ErrorSourceDomain::News
                        }) {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                        if n == 0 {
                            self.news_list_state.select(None);
                        } else {
                            let i = self
                                .news_list_state
                                .selected()
                                .unwrap_or(0)
                                .min(n - 1);
                            self.news_list_state.select(Some(i));
                        }
                    }
                    Err(err) => {
                        self.last_failed_fetch = LastFailedFetch::News {
                            symbol: symbol.clone(),
                        };
                        self.surface_runtime_error(
                            Tab::News,
                            ErrorSourceDomain::News,
                            AppError::Provider(err),
                            true,
                        );
                        self.news_data = None;
                        self.news_list_state.select(None);
                    }
                }
            }
            FetchDone::Search {
                generation,
                query,
                result,
            } => {
                self.search_refresh_inflight = false;
                if !search_result_matches_current(
                    generation,
                    self.search_request_generation,
                    &query,
                    &self.search_query,
                ) {
                    if self.active_tab == Tab::Search
                        && !self.search_query.trim().is_empty()
                        && self.provider_ready()
                    {
                        self.search_debounce_deadline = Some(Instant::now());
                    }
                    return;
                }
                match result {
                    Ok(data) => {
                        self.search_results = Some(data);
                        if matches!(self.last_failed_fetch, LastFailedFetch::Search { .. }) {
                            self.last_failed_fetch = LastFailedFetch::None;
                        }
                        if self.active_runtime_error.as_ref().is_some_and(|a| {
                            a.source_domain == ErrorSourceDomain::Search
                        }) {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                        let n = self
                            .search_results
                            .as_ref()
                            .map(|r| r.results.len())
                            .unwrap_or(0);
                        if n == 0 {
                            self.search_table_state.select(None);
                        } else {
                            let i = self
                                .search_table_state
                                .selected()
                                .unwrap_or(0)
                                .min(n - 1);
                            self.search_table_state.select(Some(i));
                        }
                    }
                    Err(e) => {
                        self.last_failed_fetch = LastFailedFetch::Search {
                            query: query.clone(),
                            generation,
                        };
                        self.surface_runtime_error(
                            Tab::Search,
                            ErrorSourceDomain::Search,
                            AppError::Provider(e),
                            true,
                        );
                        self.search_results = None;
                        self.search_table_state.select(None);
                    }
                }
            }
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let (fetch_tx, mut fetch_rx) = tokio::sync::mpsc::unbounded_channel();
        self.fetch_done_tx = Some(fetch_tx);
        let (recovery_tx, mut recovery_rx) = tokio::sync::mpsc::unbounded_channel();
        self.inflight_recovery_tx = Some(recovery_tx);
        let (url_op_tx, mut url_op_rx) = tokio::sync::mpsc::unbounded_channel();
        self.url_op_tx = Some(url_op_tx);

        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        spawn_event_thread(event_tx);

        self.request_immediate_stock_poll();

        loop {
            draw(terminal, self)?;

            if self.should_quit {
                self.session_persist_deadline = None;
                let _ = self.try_save_config_with_session();
                return Ok(());
            }

            tokio::select! {
                ev = event_rx.recv() => {
                    match ev {
                        Some(Event::Input(input)) => {
                            handle_event(self, input);
                            if self.should_fetch_ticker {
                                self.should_fetch_ticker = false;
                                self.sync_watchlist_selection_to_symbol();
                                self.request_immediate_stock_poll();
                                self.persist_session_to_disk();
                            }
                        }
                        Some(Event::Tick) => self.on_background_tick(),
                        None => {
                            // Event sender dropped (abnormal); best-effort persist like quit path.
                            self.session_persist_deadline = None;
                            let _ = self.try_save_config_with_session();
                            return Ok(());
                        }
                    }
                }
                done = fetch_rx.recv() => {
                    if let Some(msg) = done {
                        self.apply_fetch_done(msg);
                    }
                }
                recovery = recovery_rx.recv() => {
                    if let Some(kind) = recovery {
                        self.apply_inflight_recovery(kind);
                    }
                }
                url_op = url_op_rx.recv() => {
                    if let Some(msg) = url_op {
                        self.apply_url_op_done(msg);
                    }
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Aligns watchlist table selection with `symbol`. When the active filter matches no rows but
    /// the watchlist is non-empty, clears selection and leaves `symbol` unchanged (detail pane may
    /// still show a typed ticker not on the watchlist).
    pub fn sync_watchlist_selection_to_symbol(&mut self) {
        let f = self.watchlist_filter_indices();
        if f.is_empty() {
            self.watchlist_state.select(None);
            return;
        }
        if let Some(full_idx) = self.watchlist.iter().position(|s| s == &self.symbol) {
            if let Some(sel) = f.iter().position(|&i| i == full_idx) {
                self.watchlist_state.select(Some(sel));
                return;
            }
        }
        // Symbol not visible under the current filter — align highlight + active symbol to first match.
        self.watchlist_state.select(Some(0));
        let actual = f[0];
        if self.symbol != self.watchlist[actual] {
            self.symbol = self.watchlist[actual].clone();
            self.on_active_symbol_changed_for_charts();
            self.notify_symbol_changed_for_news();
            self.persist_session_to_disk();
        }
    }

    pub fn add_current_to_watchlist(&mut self) {
        let prev_effective = self.symbol.clone();
        let Some(sym) = normalize_symbol(&self.symbol) else {
            return;
        };
        if self.watchlist.iter().any(|s| s == &sym) {
            return;
        }
        let same_ticker_case_only = prev_effective.eq_ignore_ascii_case(&sym);
        self.watchlist.push(sym.clone());
        self.symbol = sym;
        self.config.watchlist = self.watchlist.clone();
        if let Err(e) = self.try_save_config_with_session() {
            self.surface_runtime_error(
                Tab::StockView,
                ErrorSourceDomain::Portfolio,
                AppError::ConfigSave(format!("Failed to save watchlist: {e}")),
                true,
            );
        } else if self.active_runtime_error.as_ref().is_some_and(|a| {
            a.source_domain == ErrorSourceDomain::Portfolio
        }) {
            self.active_runtime_error = None;
        }
        let f = self.watchlist_filter_indices();
        let new_last = self.watchlist.len().saturating_sub(1);
        if f.is_empty() {
            self.watchlist_state.select(None);
        } else if let Some(sel) = f.iter().position(|&i| i == new_last) {
            self.watchlist_state.select(Some(sel));
        } else {
            // New row is not in the current filtered view — keep `symbol` as the added ticker.
            self.watchlist_state.select(None);
        }
        if !same_ticker_case_only {
            self.on_active_symbol_changed_for_charts();
        }
        self.notify_symbol_changed_for_news();
    }

    pub fn remove_selected_watchlist_row(&mut self) {
        let Some(sel_f) = self.watchlist_state.selected() else {
            return;
        };
        let f = self.watchlist_filter_indices();
        if sel_f >= f.len() {
            return;
        }
        let actual = f[sel_f];
        if actual >= self.watchlist.len() {
            return;
        }
        self.watchlist.remove(actual);
        self.watchlist_quotes.retain(|k, _| self.watchlist.contains(k));
        self.config.watchlist = self.watchlist.clone();
        if let Err(e) = self.try_save_config_with_session() {
            self.surface_runtime_error(
                Tab::StockView,
                ErrorSourceDomain::Portfolio,
                AppError::ConfigSave(format!("Failed to save watchlist: {e}")),
                true,
            );
        } else if self.active_runtime_error.as_ref().is_some_and(|a| {
            a.source_domain == ErrorSourceDomain::Portfolio
        }) {
            self.active_runtime_error = None;
        }

        if self.watchlist.is_empty() {
            self.watchlist_state.select(None);
        } else {
            let f2 = self.watchlist_filter_indices();
            if f2.is_empty() {
                self.watchlist_state.select(None);
                // Filter matches no rows but tickers remain — keep `symbol` in the watchlist.
                self.symbol = self.watchlist[0].clone();
                self.persist_session_to_disk();
            } else {
                let ni = sel_f.min(f2.len().saturating_sub(1));
                self.watchlist_state.select(Some(ni));
                self.symbol = self.watchlist[f2[ni]].clone();
            }
        }

        self.ticker_data = self
            .watchlist_quotes
            .get(&self.symbol)
            .cloned();
        if !self.watchlist.is_empty() {
            self.on_active_symbol_changed_for_charts();
        }
        self.notify_symbol_changed_for_news();
    }

    pub fn watchlist_select_prev(&mut self) {
        let f = self.watchlist_filter_indices();
        if f.is_empty() {
            return;
        }
        match self.watchlist_state.selected() {
            None => self
                .watchlist_state
                .select(Some(f.len().saturating_sub(1))),
            Some(i) if i > 0 => self.watchlist_state.select(Some(i - 1)),
            _ => {}
        }
        if let Some(i) = self.watchlist_state.selected() {
            if i < f.len() {
                self.symbol = self.watchlist[f[i]].clone();
                self.on_active_symbol_changed_for_charts();
            }
        }
        self.notify_symbol_changed_for_news();
        self.persist_session_to_disk();
    }

    pub fn watchlist_select_next(&mut self) {
        let f = self.watchlist_filter_indices();
        if f.is_empty() {
            return;
        }
        match self.watchlist_state.selected() {
            None => self.watchlist_state.select(Some(0)),
            Some(i) if i < f.len().saturating_sub(1) => {
                self.watchlist_state.select(Some(i + 1));
            }
            _ => {}
        }
        if let Some(i) = self.watchlist_state.selected() {
            if i < f.len() {
                self.symbol = self.watchlist[f[i]].clone();
                self.on_active_symbol_changed_for_charts();
            }
        }
        self.notify_symbol_changed_for_news();
        self.persist_session_to_disk();
    }

    /// Charts tab: switch time range and refetch (keys `1`–`4`).
    ///
    /// Selecting the **same** range again still bypasses the charts throttle and resets the
    /// viewport so e.g. **`3` on default 1M** forces a refresh (auditor: no early-return no-op).
    pub fn set_charts_time_range(&mut self, tr: TimeRange) {
        let changed = self.time_range != tr;
        if changed {
            self.time_range = tr;
            self.historical_data = None;
            self.chart_viewport = ChartViewport::default();
            if self.active_runtime_error.as_ref().is_some_and(|a| {
                a.source_domain == ErrorSourceDomain::Charts
            }) {
                self.active_runtime_error = None;
            }
        }
        self.request_immediate_charts_poll();
        if !changed {
            self.charts_reset_viewport();
        }
    }

    /// Bypass charts throttle (e.g. after changing `time_range`).
    pub fn request_immediate_charts_poll(&mut self) {
        self.last_charts_network_poll = None;
    }

    pub fn charts_zoom_in(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        viewport_zoom_in(&mut self.chart_viewport, h.results.len());
    }

    pub fn charts_zoom_out(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        viewport_zoom_out(&mut self.chart_viewport, h.results.len());
    }

    pub fn charts_pan_left(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        crate::app::charts::viewport_pan_left(&mut self.chart_viewport, h.results.len());
    }

    pub fn charts_pan_right(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        crate::app::charts::viewport_pan_right(&mut self.chart_viewport, h.results.len());
    }

    pub fn charts_reset_viewport(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        self.chart_viewport = ChartViewport::full(h.results.len());
    }

    pub fn charts_toggle_mode(&mut self) {
        self.chart_mode = self.chart_mode.toggle();
    }

    pub fn next_tab(&mut self) {
        let from = self.active_tab;
        self.active_tab = match self.active_tab {
            Tab::StockView => Tab::Portfolio,
            Tab::Portfolio => Tab::Alerts,
            Tab::Alerts => Tab::Search,
            Tab::Search => Tab::News,
            Tab::News => Tab::Charts,
            Tab::Charts => Tab::Settings,
            Tab::Settings => Tab::StockView,
        };
        if from == Tab::Portfolio && self.active_tab != Tab::Portfolio {
            self.clear_portfolio_tab_transient();
        }
        self.clear_table_filter();
        self.persist_session_to_disk();
    }

    pub fn prev_tab(&mut self) {
        let from = self.active_tab;
        self.active_tab = match self.active_tab {
            Tab::StockView => Tab::Settings,
            Tab::Portfolio => Tab::StockView,
            Tab::Alerts => Tab::Portfolio,
            Tab::Search => Tab::Alerts,
            Tab::News => Tab::Search,
            Tab::Charts => Tab::News,
            Tab::Settings => Tab::Charts,
        };
        if from == Tab::Portfolio && self.active_tab != Tab::Portfolio {
            self.clear_portfolio_tab_transient();
        }
        self.clear_table_filter();
        self.persist_session_to_disk();
    }

    /// Adds or merges a holding for the active [`Self::symbol`] and persists config.
    ///
    /// # Returns
    ///
    /// - `true` if the holding was applied and [`Self::try_save_config_with_session`] succeeded.
    /// - `false` if:
    ///   1. [`crate::app::normalize_symbol`] on [`Self::symbol`] is `None` — does **not** set
    ///      `error_message`; caller ([`crate::app::portfolio::try_commit_portfolio_dialog`]) must set
    ///      `portfolio_dialog.inline_error`.
    ///   2. config save fails — sets runtime error via [`Self::surface_runtime_error`]; caller must
    ///      **not** overwrite with `inline_error`.
    ///
    /// Any new `false` branch must either set `error_message` or extend the contract in SPEC §36.3.
    pub fn add_to_portfolio(&mut self, shares: f64, purchase_price: f64) -> bool {
        let Some(sym) = normalize_symbol(&self.symbol) else {
            return false;
        };

        let backup = self.portfolio.clone();

        if let Some(item) = self
            .portfolio
            .iter_mut()
            .find(|i| normalize_symbol(&i.symbol).as_deref() == Some(sym.as_str()))
        {
            item.shares += shares;
            let total_shares = item.shares;
            let existing_cost = (total_shares - shares) * item.purchase_price;
            let new_cost = shares * purchase_price;
            item.purchase_price = (existing_cost + new_cost) / total_shares;
        } else {
            self.portfolio
                .push(PortfolioItem::new(sym.clone(), shares, purchase_price));
        }

        self.config.portfolio = self.portfolio.clone();
        match self.try_save_config_with_session() {
            Ok(()) => {
                if !self.portfolio.is_empty() {
                    let f = self.portfolio_filter_indices();
                    if !f.is_empty() {
                        let pos = f
                            .iter()
                            .position(|&i| {
                                normalize_symbol(&self.portfolio[i].symbol).as_deref()
                                    == Some(sym.as_str())
                            })
                            .unwrap_or(f.len().saturating_sub(1));
                        self.portfolio_state.select(Some(pos));
                    }
                } else {
                    self.portfolio_state.select(None);
                }
                true
            }
            Err(e) => {
                self.portfolio = backup;
                self.config.portfolio = self.portfolio.clone();
                self.surface_runtime_error(
                    Tab::Portfolio,
                    ErrorSourceDomain::Portfolio,
                    AppError::ConfigSave(e.to_string()),
                    true,
                );
                false
            }
        }
    }

    /// Returns `false` if index invalid or `try_save` failed.
    pub fn remove_from_portfolio(&mut self, index: usize) -> bool {
        if index >= self.portfolio.len() {
            return false;
        }

        let backup = self.portfolio.clone();
        self.portfolio.remove(index);
        self.config.portfolio = self.portfolio.clone();
        match self.try_save_config_with_session() {
            Ok(()) => {
                self.clamp_portfolio_filter_selection();
                true
            }
            Err(e) => {
                self.portfolio = backup;
                self.config.portfolio = self.portfolio.clone();
                self.surface_runtime_error(
                    Tab::Portfolio,
                    ErrorSourceDomain::Portfolio,
                    AppError::ConfigSave(e.to_string()),
                    true,
                );
                false
            }
        }
    }

    pub fn calculate_portfolio_value(&self) -> f64 {
        self.portfolio
            .iter()
            .filter_map(|item| item.market_value())
            .sum()
    }

    pub fn calculate_portfolio_cost(&self) -> f64 {
        self.portfolio.iter().map(|item| item.cost_basis()).sum()
    }

    pub fn calculate_portfolio_profit_loss(&self) -> f64 {
        self.calculate_portfolio_value() - self.calculate_portfolio_cost()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        data_poll_interval_secs, normalize_symbol, search_result_matches_current, App,
    };
    use crate::app::app_error::{push_error_log, ErrorLogEntry, UiErrorCategory, ERROR_LOG_CAP};
    use crate::app::Tab;
    use std::collections::VecDeque;

    fn fill_error_log(app: &mut App, n: usize) {
        for i in 0..n {
            push_error_log(
                &mut app.error_log,
                Tab::StockView,
                UiErrorCategory::Int,
                format!("msg{i}"),
            );
        }
    }

    /// Issue #120 / SPEC §20.15.6 — clamp brings out-of-range scroll back to
    /// `total - visible` when there are more entries than visible rows.
    #[test]
    fn clamp_error_log_scroll_clamps_to_total_minus_visible() {
        let mut app = App::new();
        fill_error_log(&mut app, 30);
        // Ring caps at 20.
        assert_eq!(app.error_log.len(), ERROR_LOG_CAP);

        app.error_log_visible_rows = 5;
        app.error_log_scroll = 99;
        app.clamp_error_log_scroll();
        assert_eq!(app.error_log_scroll, ERROR_LOG_CAP - 5);
    }

    /// Issue #120 / SPEC §20.15.6 — when the visible viewport can show every
    /// entry, scroll snaps to 0 (no off-the-end window).
    #[test]
    fn clamp_error_log_scroll_visible_exceeds_total_resets_to_zero() {
        let mut app = App::new();
        fill_error_log(&mut app, 5);
        app.error_log_visible_rows = 100;
        app.error_log_scroll = 4;
        app.clamp_error_log_scroll();
        assert_eq!(app.error_log_scroll, 0);
    }

    /// Issue #120 / SPEC §20.15.6 — empty log + tiny viewport must not
    /// underflow `usize` arithmetic.
    #[test]
    fn clamp_error_log_scroll_empty_log_no_underflow() {
        let mut app = App::new();
        app.error_log = VecDeque::new();
        app.error_log_visible_rows = 1;
        app.error_log_scroll = 7;
        app.clamp_error_log_scroll();
        assert_eq!(app.error_log_scroll, 0);
    }

    /// Issue #121 / SPEC §20.15.6 — calling `clamp_error_log_scroll` twice in
    /// a row produces no further change (idempotent helper safe to invoke
    /// from both input handlers and the `Ctrl+E` open path).
    #[test]
    fn clamp_error_log_scroll_is_idempotent() {
        let mut app = App::new();
        fill_error_log(&mut app, 12);
        app.error_log_visible_rows = 4;
        app.error_log_scroll = 50;
        app.clamp_error_log_scroll();
        let first = app.error_log_scroll;
        app.clamp_error_log_scroll();
        assert_eq!(app.error_log_scroll, first);
        assert_eq!(first, 12 - 4);
    }

    /// Issue #120 / #121 / SPEC §20.15.2 (audit follow-up) — when the user is
    /// scrolled to the bottom of the overlay and a new error pushes (the ring
    /// evicts the oldest entry), `clamp_error_log_scroll` must keep the
    /// scroll anchored to the new bottom so the next `k` press actually
    /// moves the visible window. Regression guard for the dead-key bug.
    #[test]
    fn push_error_log_then_clamp_keeps_bottom_anchored() {
        let mut app = App::new();
        // Exactly fill the ring so the next push triggers an eviction.
        fill_error_log(&mut app, ERROR_LOG_CAP);
        assert_eq!(app.error_log.len(), ERROR_LOG_CAP);

        let visible = 5;
        app.error_log_visible_rows = visible;
        let bottom = ERROR_LOG_CAP - visible;
        app.error_log_scroll = bottom;

        // Simulate a single new error landing through the App-level helper:
        // mimics the `surface_runtime_error` / `apply_stock_fetch_done` flow.
        push_error_log(
            &mut app.error_log,
            Tab::StockView,
            UiErrorCategory::Int,
            "newest".into(),
        );
        app.clamp_error_log_scroll();

        // Total stays at CAP; the new bottom is unchanged numerically, and
        // crucially `error_log_scroll` is still a valid `max_scroll` (not
        // stale by +1), so the next `k` press will scroll up by exactly one
        // row instead of being a no-op.
        assert_eq!(app.error_log.len(), ERROR_LOG_CAP);
        let max_scroll = app.error_log.len() - visible;
        assert!(
            app.error_log_scroll <= max_scroll,
            "scroll {} should be <= max_scroll {}",
            app.error_log_scroll,
            max_scroll
        );

        // Simulate the input handler's `k` step.
        app.error_log_scroll = app.error_log_scroll.saturating_sub(1);
        assert_eq!(
            app.error_log_scroll,
            max_scroll - 1,
            "first `k` after eviction must move the window up by exactly one row"
        );
    }

    /// Issue #120 / #121 / SPEC §20.15.1 (round-2 audit follow-up) — terminal
    /// resize-larger shrinks the layout-derived `max_scroll` but not
    /// `error_log_scroll`. Without an entry-clamp in
    /// `handle_error_log_overlay_keys`, the next `k` press is a no-op (the
    /// local-clamp in `draw_error_log_overlay` masks the staleness for
    /// *rendering* only, while the input handler's `saturating_sub` operates
    /// on the stale field). This test would have failed against the
    /// pre-round-2 implementation. Drives `handle_event` end-to-end so the
    /// `q`-quit / `Ctrl+E` / overlay-routing layers stay covered.
    #[test]
    fn resize_larger_does_not_strand_k_against_stale_scroll() {
        use crate::app::handlers::handle_event;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new();
        fill_error_log(&mut app, ERROR_LOG_CAP);

        // Pre-resize: small viewport, user scrolled to the bottom.
        let pre_visible = 5;
        app.error_log_visible_rows = pre_visible;
        app.error_log_scroll = ERROR_LOG_CAP - pre_visible;
        app.error_log_overlay_open = true;

        // Simulate `draw_error_log_overlay` running at a *larger* terminal
        // size: it publishes the new `error_log_visible_rows` but, per Issue
        // #121, must NOT touch `error_log_scroll`. We mimic that here.
        let post_visible = 18;
        app.error_log_visible_rows = post_visible;
        let post_max_scroll = ERROR_LOG_CAP - post_visible;
        assert_eq!(post_max_scroll, 2, "test arithmetic sanity");

        // One `k` keystroke through the real input pipeline.
        handle_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
        );

        assert!(
            app.error_log_scroll <= post_max_scroll,
            "scroll {} must be <= post-resize max_scroll {}",
            app.error_log_scroll,
            post_max_scroll,
        );
        assert_eq!(
            app.error_log_scroll,
            post_max_scroll - 1,
            "first `k` after resize-larger must scroll up by exactly one painted row"
        );
        // Sanity: `q` quit and other globals didn't fire.
        assert!(!app.should_quit);
        assert!(app.error_log_overlay_open);
    }

    /// Issue #120 / SPEC §20.15.1 — `error_log_visible_rows` defaults to a
    /// non-zero floor so the very first key press after `Ctrl+E` cannot
    /// divide-by-zero or trip a `saturating_sub` to a misleading value.
    #[test]
    fn error_log_visible_rows_initial_floor_is_nonzero() {
        let app = App::new();
        assert!(app.error_log_visible_rows >= 1);
        // Sanity: if a future refactor lowers the floor, the helper still
        // treats `0` as `1` to preserve the contract.
        let mut app2 = app;
        app2.error_log_visible_rows = 0;
        let mut e: VecDeque<ErrorLogEntry> = VecDeque::new();
        push_error_log(&mut e, Tab::StockView, UiErrorCategory::Int, "x".into());
        app2.error_log = e;
        app2.error_log_scroll = 5;
        app2.clamp_error_log_scroll();
        // total = 1, visible_rows treated as 1 → max_scroll = 0.
        assert_eq!(app2.error_log_scroll, 0);
    }

    #[test]
    fn apply_stock_fetch_done_merges_alerts_save_with_quote_errors_issue_103() {
        use crate::api::error::ProviderError;
        use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::collections::HashMap;
        use std::time::Instant;

        let mut app = App::new();
        app.symbol = "AAPL".to_string();
        app.stock_fetch_generation = 1;
        app.stock_refresh_inflight = true;
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::ConfigSave(format!("{ALERTS_SAVE_ERROR_PREFIX} simulated")),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Alerts,
        ));

        let errors = vec![("AAPL".into(), ProviderError::ApiMessage("bad".into()))];
        app.apply_stock_fetch_done(1, HashMap::new(), errors);
        let msg = app.error_message().expect("merged error");
        assert!(
            msg.contains(ALERTS_SAVE_ERROR_PREFIX),
            "status line should keep alerts-save prefix: {msg}"
        );
        assert!(msg.contains("bad"), "expected quote detail: {msg}");
    }

    #[test]
    fn apply_stock_fetch_done_remerges_when_active_error_is_internal_audit_round2() {
        use crate::api::error::ProviderError;
        use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::collections::HashMap;
        use std::time::Instant;

        let mut app = App::new();
        app.symbol = "AAPL".to_string();
        app.stock_fetch_generation = 1;
        app.stock_refresh_inflight = true;
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::Internal(format!(
                "{ALERTS_SAVE_ERROR_PREFIX} disk · first-batch"
            )),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Stock,
        ));

        let errors = vec![("MSFT".into(), ProviderError::ApiMessage("second".into()))];
        app.apply_stock_fetch_done(1, HashMap::new(), errors);
        let msg = app.error_message().expect("second merge");
        assert!(
            msg.contains(ALERTS_SAVE_ERROR_PREFIX),
            "must keep alerts prefix: {msg}"
        );
        assert!(msg.contains("second"), "second batch detail: {msg}");
    }

    #[test]
    fn tab_config_str_roundtrip() {
        assert_eq!(Tab::Charts.as_config_str(), "charts");
        assert_eq!(Tab::from_config_str("charts"), Some(Tab::Charts));
        assert_eq!(Tab::from_config_str("Charts"), Some(Tab::Charts));
        assert!(Tab::from_config_str("nope").is_none());
    }

    #[test]
    fn clear_alerts_save_runtime_error_clears_merged_internal_issue_audit() {
        use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::time::Instant;

        let mut app = App::new();
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::Internal(format!(
                "{ALERTS_SAVE_ERROR_PREFIX} simulated · API: bad"
            )),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Stock,
        ));
        app.clear_alerts_save_runtime_error_after_recovery();
        assert!(
            app.active_runtime_error.is_none(),
            "merged §22.2 Internal line must clear after alerts save recovery"
        );
    }

    #[test]
    fn normalize_symbol_trims_and_uppercases() {
        assert_eq!(
            normalize_symbol("  aapl  ").as_deref(),
            Some("AAPL")
        );
        assert_eq!(normalize_symbol("   "), None);
        assert_eq!(normalize_symbol(""), None);
    }

    #[test]
    fn search_result_matches_current_requires_gen_and_query() {
        assert!(search_result_matches_current(1, 1, "appl", "appl"));
        assert!(!search_result_matches_current(1, 2, "appl", "appl"));
        assert!(!search_result_matches_current(1, 1, "ap", "appl"));
    }

    #[test]
    fn search_esc_reset_preserves_stock_runtime_error() {
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::time::Instant;

        let mut app = App::new();
        app.search_query = "AAPL".into();
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::Internal("quote failed".into()),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Stock,
        ));

        app.search_esc_reset();

        assert!(app.search_query.is_empty());
        assert!(app.search_results.is_none());
        assert_eq!(
            app.error_message().as_deref(),
            Some("[int] quote failed")
        );
    }

    #[test]
    fn search_esc_reset_clears_search_domain_error() {
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::time::Instant;

        let mut app = App::new();
        app.search_query = "ZZ".into();
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::Internal("search failed".into()),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Search,
        ));

        app.search_esc_reset();

        assert!(app.search_query.is_empty());
        assert!(app.error_message().is_none());
    }

    #[test]
    fn search_esc_reset_preserves_alerts_save_banner() {
        use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::time::Instant;

        let mut app = App::new();
        app.search_query = "X".into();
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::ConfigSave(format!("{ALERTS_SAVE_ERROR_PREFIX} simulated")),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Alerts,
        ));

        app.search_esc_reset();

        assert!(app.search_query.is_empty());
        assert!(app.preserves_alerts_save_banner());
        assert!(
            app.error_message()
                .is_some_and(|m| m.contains(ALERTS_SAVE_ERROR_PREFIX))
        );
    }

    #[test]
    fn collect_symbols_for_quote_includes_portfolio_only_tickers() {
        use crate::models::portfolio::PortfolioItem;

        let mut app = App::new();
        app.watchlist.clear();
        app.symbol = "AAPL".to_string();
        app.portfolio = vec![PortfolioItem::new("IBM".to_string(), 1.0, 100.0)];
        let syms = app.collect_symbols_for_quote_fetch();
        assert!(syms.contains(&"AAPL".to_string()));
        assert!(syms.contains(&"IBM".to_string()));
    }

    /// Issue #83 / SPEC §36.3 — `add_to_portfolio` false without runtime error ⇒ `inline_error`.
    #[test]
    fn portfolio_try_commit_sets_inline_error_when_add_fails_without_try_save() {
        use crate::app::portfolio::{
            try_commit_portfolio_dialog, PORTFOLIO_ADD_INVALID_SYMBOL_INLINE,
        };
        use crate::app::{PortfolioAddDialog, PortfolioAddField};

        let mut app = App::new();
        app.symbol.clear();
        app.active_runtime_error = None;
        app.portfolio_dialog = Some(PortfolioAddDialog {
            shares_buffer: "1".into(),
            price_buffer: "1".into(),
            focused: PortfolioAddField::Price,
            inline_error: None,
        });
        try_commit_portfolio_dialog(&mut app);
        assert!(app.portfolio_dialog.is_some());
        assert_eq!(
            app.portfolio_dialog
                .as_ref()
                .and_then(|d| d.inline_error.as_deref()),
            Some(PORTFOLIO_ADD_INVALID_SYMBOL_INLINE)
        );
        assert!(app.active_runtime_error.is_none());
    }

    /// Issue #4 / SPEC §35.4 — JSON default `refresh_rate: 0` maps to 30 s effective poll.
    #[test]
    fn data_poll_interval_zero_means_thirty_seconds() {
        assert_eq!(data_poll_interval_secs(0), 30);
    }

    /// Issue #4 / SPEC §35.4 — values below 5 clamp to the API-safe floor.
    #[test]
    fn data_poll_interval_enforces_five_second_floor() {
        assert_eq!(data_poll_interval_secs(1), 5);
        assert_eq!(data_poll_interval_secs(4), 5);
    }

    /// Issue #4 / SPEC §35.4 — configured values at or above the floor are honored.
    #[test]
    fn data_poll_interval_honors_configured_value() {
        assert_eq!(data_poll_interval_secs(5), 5);
        assert_eq!(data_poll_interval_secs(60), 60);
    }
}

/// Effective network poll interval in seconds for [`Config::refresh_rate`] (Issue #4 / SPEC §35.4).
///
/// `0` (unset JSON default) → 30 s; values below 5 clamp to 5.
pub(crate) fn data_poll_interval_secs(refresh_rate: u64) -> u64 {
    let secs = match refresh_rate {
        0 => 30,
        s => s,
    };
    secs.max(5)
}

/// Stale-guard for `FetchDone::Search` (SPEC §10.2).
pub(crate) fn search_result_matches_current(
    response_generation: u64,
    app_generation: u64,
    response_query: &str,
    app_query: &str,
) -> bool {
    response_generation == app_generation && response_query == app_query
}
