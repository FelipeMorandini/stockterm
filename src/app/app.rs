use crate::api::concurrency::acquire_quote_permit;
use crate::api::error::ProviderError;
use crate::api::http::maybe_debug_http_delay;
use crate::api::market_provider_for;
use crate::api::symbol::resolve_provider_symbol;
use crate::api::HistoricalQuery;
use crate::app::alerts::ALERTS_SAVE_ERROR_PREFIX;
use crate::app::app_error::{
    persistence_for_app_error, push_error_log, ActiveErrorState, AppError, ErrorLogEntry,
    ErrorPersistence, ErrorSourceDomain, LastFailedFetch, ERROR_TRANSIENT_TTL,
};
use crate::app::charts::{
    viewport_zoom_in, viewport_zoom_out, ChartCandleLayoutCache, ChartDisplayMode,
    ChartIndicatorCache, ChartIndicatorToggles, ChartViewport,
};
use crate::app::event::{join_event_thread, spawn_event_thread, Event};
use crate::app::fetch_delivery::deliver_fetch_done;
use crate::app::handlers::handle_event;
use crate::app::ui::draw;
use crate::config::keymap::{Action, BindingLayer};
use crate::config::theme::{PaletteRgb, Theme, ThemePreset};
use crate::config::{
    Config, ConfigError, LayoutPreset, MarketProviderKind, ResolvedKeymap, ResolvedLayout,
};
use crate::models::alerts::{Alert, AlertCondition};
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::portfolio::PortfolioItem;
use crate::models::search::SymbolSearchResponse;
use crate::models::symbol::{classify_from_instrument_type, classify_symbol_with_hint, SymbolKind};
use crate::models::ticker::TickerResponse;
use crate::models::time_range::TimeRange;
use futures_util::future::FutureExt;
use ratatui::backend::Backend;
use ratatui::widgets::{ListState, TableState};
use ratatui::Terminal;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;
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
    Backtest,
    Options,
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
            Tab::Backtest => "backtest",
            Tab::Options => "options",
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
            "backtest" | "Backtest" => Tab::Backtest,
            "options" | "Options" => Tab::Options,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsEdit {
    RefreshRate,
    DefaultSymbol,
    BacktestCapital,
    BacktestCommission,
    BacktestSlippage,
}

/// Add-holding dialog field focus (Issue #6 / SPEC §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioAddField {
    Shares,
    Price,
}

/// Add vs edit mode for the portfolio holding modal (Issue #182 / §55).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioDialogKind {
    /// New holding for [`App::symbol`] (Issue #6 / §13).
    Add,
    /// In-place edit of an existing [`PortfolioItem`] by portfolio vec index.
    Edit { portfolio_index: usize },
}

/// In-modal state for adding or editing a portfolio row (Issue #6 / §13; Issue #182 / §55).
#[derive(Debug, Clone)]
pub struct PortfolioAddDialog {
    pub kind: PortfolioDialogKind,
    pub shares_buffer: String,
    pub price_buffer: String,
    pub focused: PortfolioAddField,
    pub inline_error: Option<String>,
    /// Edit-only: first Enter on Price arms save; second Enter/y commits (§55.1).
    pub commit_armed: bool,
}

impl Default for PortfolioAddDialog {
    fn default() -> Self {
        Self {
            kind: PortfolioDialogKind::Add,
            shares_buffer: String::new(),
            price_buffer: String::new(),
            focused: PortfolioAddField::Shares,
            inline_error: None,
            commit_armed: false,
        }
    }
}

impl PortfolioAddDialog {
    /// Prefilled edit dialog for an existing holding (Issue #182 / §55).
    pub fn for_edit(
        item: &crate::models::portfolio::PortfolioItem,
        portfolio_index: usize,
    ) -> Self {
        use crate::app::portfolio::format_holding_input_value;
        Self {
            kind: PortfolioDialogKind::Edit { portfolio_index },
            shares_buffer: format_holding_input_value(item.shares),
            price_buffer: format_holding_input_value(item.purchase_price),
            focused: PortfolioAddField::Shares,
            inline_error: None,
            commit_armed: false,
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
    /// Backtest tab when [`FetchDone::Backtest`] could not be delivered (§47 / audit).
    Backtest,
    /// Options tab when [`FetchDone::Options`] could not be delivered (§48).
    Options,
}

/// Outcomes from background HTTP tasks (never awaited on the draw/input hot path).
pub enum FetchDone {
    Stock {
        generation: u64,
        quotes: HashMap<String, TickerResponse>,
        /// Yahoo v7 `quoteType` per normalized symbol (Issue #158 / §44.2).
        instrument_types: HashMap<String, String>,
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
    Backtest {
        result: Result<crate::models::backtest::BacktestReport, crate::backtest::BacktestError>,
    },
    Options {
        symbol: String,
        expiration_ts: Option<u64>,
        result: crate::api::error::ProviderResult<crate::models::options::OptionsChain>,
        /// Inline expiration blocks from the same HTTP response (Issue #168 / §49.2).
        extra_slices: std::collections::HashMap<u64, crate::models::options::OptionsChainSlice>,
    },
}

#[cfg(debug_assertions)]
fn log_quote_batch_panic(payload: &(dyn std::any::Any + Send)) {
    if let Some(s) = payload.downcast_ref::<&str>() {
        tracing::warn!(
            target: "stockterm::fetch",
            panic = %s,
            "quote batch task panicked"
        );
    } else if let Some(s) = payload.downcast_ref::<String>() {
        tracing::warn!(
            target: "stockterm::fetch",
            panic = %s,
            "quote batch task panicked"
        );
    } else {
        tracing::warn!(
            target: "stockterm::fetch",
            "quote batch task panicked (non-string payload)"
        );
    }
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
            if let Err(_e) = tx.send(InflightRecovery::NewsUrlOp) {
                tracing::warn!(
                    target: "stockterm::fetch",
                    kind = "news_url_op",
                    "inflight recovery send failed"
                );
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
    /// In-memory normalized symbol → [`SymbolKind`] from Search `type_` or v7 `quoteType` (Issue #158 / §44.2).
    ///
    /// Not persisted to `~/.stockterm.json`; repopulated from Search results and quote batches.
    pub symbol_kind_cache: HashMap<String, crate::models::symbol::SymbolKind>,
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
    /// Throttle Options-tab chain fetches (Issue #22 / §48.3).
    last_options_network_poll: Option<Instant>,
    /// True while a watchlist / quote batch is in flight.
    pub stock_refresh_inflight: bool,
    stock_inflight_since: Option<Instant>,
    fetch_done_tx: Option<UnboundedSender<FetchDone>>,
    inflight_recovery_tx: Option<UnboundedSender<InflightRecovery>>,
    stock_fetch_generation: u64,
    stock_refresh_pending: bool,
    hist_refresh_inflight: bool,
    hist_inflight_since: Option<Instant>,
    pub news_refresh_inflight: bool,
    news_inflight_since: Option<Instant>,
    search_inflight_since: Option<Instant>,
    url_op_tx: Option<UnboundedSender<UrlOpDone>>,
    news_url_op_inflight: bool,
    news_url_flash: Option<(crate::app::open_url::NewsUrlFlashHint, Instant)>,
    /// Charts tab: selected window (Issue #9).
    pub time_range: TimeRange,
    /// Charts tab: pan/zoom indices into `historical_data.results` (Issue #8).
    pub chart_viewport: ChartViewport,
    /// Polygon partial-page chart notice (Issue #65 / §52.1.3).
    pub charts_polygon_truncated: bool,
    /// Precomputed Charts status suffix when [`charts_polygon_truncated`] (Update only).
    pub charts_polygon_notice: String,
    /// Line vs candlestick rendering (Issue #7).
    pub chart_mode: ChartDisplayMode,
    /// Charts tab: SMA/EMA/RSI/MACD toggles (Issue #21 / §46.2).
    pub chart_indicators: ChartIndicatorToggles,
    /// Precomputed indicators for `historical_data` (rebuilt on fetch/toggle).
    pub(crate) chart_indicator_cache: Option<ChartIndicatorCache>,
    /// Issue #199 / §64 — precomputed candle body layout for the current Charts draw.
    pub(crate) chart_candle_layout: Option<ChartCandleLayoutCache>,
    /// Monotonically incremented on every successful `apply_stock_fetch_done` historical apply;
    /// also bumped on `clear_active_symbol_data` so stale caches invalidate. Used as `series_stamp`.
    pub historical_data_stamp: u64,
    /// Last backtest result (session-only; Issue #25 / §47.3).
    pub backtest_report: Option<crate::models::backtest::BacktestReport>,
    pub backtest_inflight: bool,
    backtest_inflight_since: Option<Instant>,
    pub backtest_trade_list_state: ratatui::widgets::TableState,
    pub backtest_flash: Option<(String, Instant)>,
    /// Precomputed Backtest tab table/chart data (Issue #25 — not built in `draw`).
    pub(crate) backtest_draw_cache: Option<crate::app::backtest_ui::BacktestDrawCache>,
    /// Precomputed Backtest left-pane parameter lines (Issue #25 — not built in `draw`).
    pub(crate) backtest_params_cache: Option<crate::app::backtest_ui::BacktestParamsCache>,
    /// Options chain for active symbol (Issue #22 / §48.3).
    pub options_chain: Option<crate::models::options::OptionsChain>,
    pub options_inflight: bool,
    options_inflight_since: Option<Instant>,
    /// Highlighted strike shared across calls/puts tables (Issue #22).
    pub options_selected_strike: Option<f64>,
    pub options_show_greeks: bool,
    /// True after provider reports no listed options for symbol.
    pub options_no_listed: bool,
    pub options_display: crate::app::options::OptionsDisplayCache,
    /// Options CALLS table scroll/selection (Issue #177 / §53.2).
    pub options_calls_table_state: TableState,
    /// Options PUTS table scroll/selection (Issue #177 / §53.2).
    pub options_puts_table_state: TableState,
    /// Per-expiration chain slices for the active symbol (session-only, Issue #168).
    pub options_slices_by_ts:
        std::collections::HashMap<u64, crate::models::options::OptionsChainSlice>,
    /// Polygon wire symbol + deduped expiration list (session-only, Issue #171 / §51).
    pub options_polygon_expirations_cache:
        Option<(String, Vec<crate::models::options::Expiration>)>,
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
pub const SETTINGS_ROW_COUNT: usize = 10;

const SETTINGS_SAVED_FLASH: Duration = Duration::from_secs(2);

const NEWS_URL_FLASH: Duration = Duration::from_secs(2);

/// Issue #78 / SPEC §39.2 — clear stuck inflight when both channel sends fail.
const INFLIGHT_STALE_AFTER: Duration = Duration::from_secs(120);

fn inflight_stale_after() -> Duration {
    std::env::var("STOCKTERM_INFLIGHT_STALE_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(INFLIGHT_STALE_AFTER)
}

pub use crate::models::symbol::{normalize_symbol, symbols_equivalent};

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

async fn run_stock_quote_batch(generation: u64, symbols: Vec<String>, config: Config) -> FetchDone {
    maybe_debug_http_delay().await;

    if config.provider == MarketProviderKind::Yahoo {
        let (quotes_raw, instrument_types, mut errors) =
            crate::api::yahoo::yahoo_latest_quotes_for_symbols(&symbols, MAX_CONCURRENT_QUOTES)
                .await;
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
            instrument_types,
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
            let _permit = match acquire_quote_permit(&sem, &sym, "polygon").await {
                Ok(p) => p,
                Err(e) => return (sym, Err(e)),
            };
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
        instrument_types: HashMap::new(),
        errors,
    }
}

#[cfg(test)]
thread_local! {
    static CANDLE_LAYOUT_BUILD_COUNTER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Resets the per-test-thread layout rebuild counter (§64.6; parallel-safe).
#[cfg(test)]
pub(crate) fn test_reset_candle_layout_build_counter() {
    CANDLE_LAYOUT_BUILD_COUNTER.with(|c| c.set(0));
}

#[cfg(test)]
pub(crate) fn test_candle_layout_build_count() -> usize {
    CANDLE_LAYOUT_BUILD_COUNTER.with(|c| c.get())
}

impl App {
    pub fn new() -> App {
        let (config, mut startup_error) = match Config::try_load() {
            Ok(c) => (c, None),
            Err(e) => (
                Config::default(),
                Some(AppError::ConfigSave(format!("Config load failed: {e}"))),
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

        let time_range = config
            .last_time_range
            .as_deref()
            .and_then(TimeRange::from_config_str)
            .unwrap_or_default();
        let chart_mode = config
            .last_chart_mode
            .as_deref()
            .and_then(ChartDisplayMode::from_config_str)
            .unwrap_or_default();

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
            symbol_kind_cache: HashMap::new(),
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
            last_options_network_poll: None,
            stock_refresh_inflight: false,
            stock_inflight_since: None,
            fetch_done_tx: None,
            inflight_recovery_tx: None,
            stock_fetch_generation: 0,
            stock_refresh_pending: false,
            hist_refresh_inflight: false,
            hist_inflight_since: None,
            news_refresh_inflight: false,
            news_inflight_since: None,
            search_inflight_since: None,
            url_op_tx: None,
            news_url_op_inflight: false,
            news_url_flash: None,
            time_range,
            chart_viewport: ChartViewport::default(),
            charts_polygon_truncated: false,
            charts_polygon_notice: String::new(),
            chart_mode,
            chart_indicators: ChartIndicatorToggles::default(),
            chart_indicator_cache: None,
            chart_candle_layout: None,
            historical_data_stamp: 0,
            backtest_report: None,
            backtest_inflight: false,
            backtest_inflight_since: None,
            backtest_trade_list_state: ratatui::widgets::TableState::default(),
            backtest_flash: None,
            backtest_draw_cache: None,
            backtest_params_cache: None,
            options_chain: None,
            options_inflight: false,
            options_inflight_since: None,
            options_selected_strike: None,
            options_show_greeks: false,
            options_no_listed: false,
            options_display: crate::app::options::OptionsDisplayCache::default(),
            options_calls_table_state: TableState::default(),
            options_puts_table_state: TableState::default(),
            options_slices_by_ts: std::collections::HashMap::new(),
            options_polygon_expirations_cache: None,
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

        crate::app::backtest_ui::rebuild_backtest_params_cache(&mut app);
        crate::app::options::sync_options_chrome(&mut app);
        app
    }

    /// Status-line text for active runtime error (Issue #103 checks this string).
    pub fn error_message(&self) -> Option<String> {
        self.active_runtime_error.as_ref().map(|a| a.display_line())
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
            push_error_log(&mut self.error_log, tab, err.category(), err.status_line());
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

    /// Copy session UI fields into `config` before any disk write (Issue #19 / §22, #180 / §54).
    pub(crate) fn sync_session_fields_into_config(&mut self) {
        self.config.last_tab = Some(self.active_tab.as_config_str().to_string());
        self.config.last_symbol = normalize_symbol(&self.symbol);
        self.config.last_time_range = Some(self.time_range.as_config_str().to_string());
        self.config.last_chart_mode = Some(self.chart_mode.as_config_str().to_string());
    }

    /// [`Config::try_save`] after refreshing session fields (`last_tab`, `last_symbol`,
    /// `last_time_range`, `last_chart_mode`) from UI state (Issues #19 / #129 / #180).
    pub(crate) fn try_save_config_with_session(&mut self) -> Result<(), ConfigError> {
        self.sync_session_fields_into_config();
        self.config.try_save()
    }

    /// Schedule a debounced flush of session fields to `~/.stockterm.json` (Issue #129 / §54).
    ///
    /// Coalesces `last_tab`, `last_symbol`, `last_time_range`, and `last_chart_mode`.
    /// High-frequency callers (`j`/`k`, tab keys, chart range/mode, Stock View **Enter**) share one write.
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

    /// Interactive persist: surface `[cfg]` on disk failure; clear same-domain sticky
    /// cfg error on success (Issue #192 / SPEC §66).
    ///
    /// Returns `true` when the write succeeded.
    pub(crate) fn persist_config_interactive(
        &mut self,
        tab: Tab,
        domain: ErrorSourceDomain,
        context: &str,
    ) -> bool {
        match self.try_save_config_with_session() {
            Ok(()) => {
                if self.active_runtime_error.as_ref().is_some_and(|a| {
                    a.source_domain == domain && matches!(a.error, AppError::ConfigSave(_))
                }) {
                    self.active_runtime_error = None;
                }
                true
            }
            Err(e) => {
                self.surface_runtime_error(
                    tab,
                    domain,
                    AppError::ConfigSave(format!("Failed to save {context}: {e}")),
                    true,
                );
                false
            }
        }
    }

    /// Final persist on quit or abnormal shutdown; log only (Issue #192 / SPEC §66).
    fn persist_config_on_shutdown(&mut self) {
        self.session_persist_deadline = None;
        if let Err(e) = self.try_save_config_with_session() {
            tracing::error!(error = %e, "final config persist failed on shutdown");
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
        self.active_runtime_error
            .as_ref()
            .and_then(|a| match &a.error {
                AppError::ConfigSave(s) if s.starts_with(ALERTS_SAVE_ERROR_PREFIX) => {
                    Some(s.as_str())
                }
                AppError::Internal(s) if s.starts_with(ALERTS_SAVE_ERROR_PREFIX) => {
                    Some(s.as_str())
                }
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
            if let Some(sym) = normalize_symbol(s) {
                if seen.insert(sym.clone()) {
                    out.push(sym);
                }
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
        crate::app::table_filter::filter_row_indices(
            self.portfolio.len(),
            |i| self.portfolio[i].symbol.as_str(),
            &self.filter_query,
        )
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

        let Some(action) = self.resolved_keymap.action(BindingLayer::FilterInput, key) else {
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
                    if (c.is_alphabetic() || c.is_ascii_digit() || c == '-' || c == '.' || c == '=')
                        && self.filter_query.len() < crate::app::table_filter::MAX_FILTER_QUERY_LEN
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
        self.stock_inflight_since = Some(Instant::now());
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
                Err(payload) => {
                    #[cfg(debug_assertions)]
                    log_quote_batch_panic(&*payload);
                    #[cfg(not(debug_assertions))]
                    drop(payload);
                    FetchDone::Stock {
                        generation,
                        quotes: HashMap::new(),
                        instrument_types: HashMap::new(),
                        errors: vec![(
                            String::new(),
                            ProviderError::ApiMessage("quote batch task panicked".into()),
                        )],
                    }
                }
            };
            deliver_fetch_done(&tx, recovery_tx.as_ref(), done, InflightRecovery::Stock);
        });
    }

    fn apply_stock_fetch_done(
        &mut self,
        generation: u64,
        quotes: HashMap<String, TickerResponse>,
        instrument_types: HashMap<String, String>,
        errors: Vec<(String, ProviderError)>,
    ) {
        // Stale batch: a newer `stock_fetch_generation` means another batch is authoritative; keep
        // `stock_refresh_inflight` unchanged (still true if a newer batch is in flight). SPEC §16.2.1.
        if generation != self.stock_fetch_generation {
            return;
        }

        self.stock_refresh_inflight = false;
        self.stock_inflight_since = None;
        self.last_stock_network_poll = Some(Instant::now());

        for (k, v) in quotes {
            self.watchlist_quotes.insert(k, v);
        }

        // Issue #160 / §45.1 — Yahoo `quoteType` only; ignore stale Yahoo batches after Polygon switch.
        if self.config.provider == MarketProviderKind::Yahoo {
            for (sym, qt) in instrument_types {
                if let Some(n) = normalize_symbol(&sym) {
                    self.remember_symbol_kind_from_instrument_type(&n, &qt);
                }
            }
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

            self.surface_runtime_error(Tab::StockView, ErrorSourceDomain::Stock, primary, false);
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
            self.invalidate_candle_layout();
            return;
        }

        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };

        self.hist_refresh_inflight = true;
        self.hist_inflight_since = Some(Instant::now());
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
                polygon_limit: params.polygon_limit,
            };
            let provider = market_provider_for(cfg.provider);
            let result = provider.get_historical(&sym, &hq, &cfg).await;
            deliver_fetch_done(
                &tx,
                recovery_tx.as_ref(),
                FetchDone::Historical {
                    symbol: sym,
                    time_range: tr,
                    result,
                },
                InflightRecovery::Historical,
            );
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
        self.news_inflight_since = Some(Instant::now());
        let sym = self.symbol.clone();
        let cfg = self.config.clone();
        let recovery_tx = self.inflight_recovery_tx.clone();
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.get_news(&sym, &cfg).await;
            deliver_fetch_done(
                &tx,
                recovery_tx.as_ref(),
                FetchDone::News {
                    symbol: sym,
                    result,
                },
                InflightRecovery::News,
            );
        });
    }

    fn on_background_tick(&mut self) {
        match self.active_tab {
            Tab::StockView | Tab::Alerts => self.try_spawn_stock_poll_throttled(),
            Tab::Charts => self.try_spawn_historical_fetch(),
            Tab::News => self.try_spawn_news_fetch(),
            Tab::Search => self.try_spawn_search_tick(),
            Tab::Options => self.try_spawn_options_fetch(),
            _ => {}
        }
        self.tick_runtime_error_ttl();
        self.flush_session_persist_if_due();
        self.recover_stale_inflight_flags();
    }

    /// Clears inflight flags left stuck when both `FetchDone` and `InflightRecovery` delivery fail.
    fn recover_stale_inflight_flags(&mut self) {
        let stale_after = inflight_stale_after();

        if self.stock_refresh_inflight
            && Self::inflight_is_stale(self.stock_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "stock",
                "cleared stale inflight after channel delivery failure"
            );
            self.apply_inflight_recovery(InflightRecovery::Stock);
        }

        if self.hist_refresh_inflight
            && Self::inflight_is_stale(self.hist_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "historical",
                "cleared stale inflight after channel delivery failure"
            );
            self.hist_refresh_inflight = false;
            self.hist_inflight_since = None;
        }

        if self.news_refresh_inflight
            && Self::inflight_is_stale(self.news_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "news",
                "cleared stale inflight after channel delivery failure"
            );
            self.news_refresh_inflight = false;
            self.news_inflight_since = None;
        }

        if self.search_refresh_inflight
            && Self::inflight_is_stale(self.search_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "search",
                "cleared stale inflight after channel delivery failure"
            );
            self.search_refresh_inflight = false;
            self.search_inflight_since = None;
        }

        if self.backtest_inflight
            && Self::inflight_is_stale(self.backtest_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "backtest",
                "cleared stale inflight after channel delivery failure"
            );
            self.apply_inflight_recovery(InflightRecovery::Backtest);
        }

        if self.options_inflight
            && Self::inflight_is_stale(self.options_inflight_since, stale_after)
        {
            tracing::warn!(
                target: "stockterm::fetch",
                domain = "options",
                "cleared stale inflight after channel delivery failure"
            );
            self.apply_inflight_recovery(InflightRecovery::Options);
        }
    }

    fn inflight_is_stale(since: Option<Instant>, stale_after: Duration) -> bool {
        match since {
            Some(t) => t.elapsed() > stale_after,
            None => true,
        }
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
        self.search_inflight_since = Some(Instant::now());
        let recovery_tx = self.inflight_recovery_tx.clone();
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.search_symbols(&query, &cfg).await;
            deliver_fetch_done(
                &tx,
                recovery_tx.as_ref(),
                FetchDone::Search {
                    generation,
                    query,
                    result,
                },
                InflightRecovery::Search,
            );
        });
    }

    /// Clears Polygon partial-chart notice state (Issue #65 / §52.1.3).
    fn clear_charts_polygon_notice(&mut self) {
        self.charts_polygon_truncated = false;
        self.charts_polygon_notice.clear();
    }

    /// Updates truncation notice from the current [`historical_data`] when provider is Polygon.
    fn sync_charts_polygon_notice_from_hist(&mut self) {
        if self.config.provider != MarketProviderKind::Polygon {
            self.clear_charts_polygon_notice();
            return;
        }
        let limit = crate::models::time_range::polygon_historical_limit(self.time_range);
        let truncated = self
            .historical_data
            .as_ref()
            .is_some_and(|hist| crate::models::historical::polygon_page_truncated(hist, limit));
        if truncated {
            self.charts_polygon_truncated = true;
            self.charts_polygon_notice = "Polygon: partial chart (plan/limit)".into();
        } else {
            self.clear_charts_polygon_notice();
        }
    }

    /// Clear chart series when the active ticker changes (Issue #62 / SPEC §11.11.1).
    pub fn on_active_symbol_changed_for_charts(&mut self) {
        self.historical_data = None;
        self.historical_data_stamp = self.historical_data_stamp.saturating_add(1);
        self.invalidate_candle_layout();
        self.chart_viewport = ChartViewport::default();
        self.clear_charts_polygon_notice();
        self.chart_indicator_cache = None;
        self.last_charts_network_poll = None;
        crate::app::backtest_ui::clear_backtest_session(self);
        crate::app::backtest_ui::rebuild_backtest_params_cache(self);
        crate::app::options::clear_options_session(self);
        self.last_options_network_poll = None;
    }

    /// Rebuild indicator cache from the current historical close series (Issue #21 / §46.2).
    pub(crate) fn rebuild_chart_indicator_cache(&mut self) {
        if !self.chart_indicators.any_enabled() {
            self.chart_indicator_cache = None;
            return;
        }
        let Some(h) = self.historical_data.as_ref() else {
            self.chart_indicator_cache = None;
            return;
        };
        if h.results.is_empty() {
            self.chart_indicator_cache = None;
            return;
        }
        let closes: Vec<f64> = h.results.iter().map(|b| b.c).collect();
        self.chart_indicator_cache = Some(ChartIndicatorCache::from_closes(&closes));
    }

    fn sync_chart_indicator_cache_after_toggle(&mut self) {
        if self.chart_indicators.any_enabled() {
            self.rebuild_chart_indicator_cache();
        } else {
            self.chart_indicator_cache = None;
        }
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
        let i = self.search_table_state.selected().unwrap_or(0).min(n - 1);
        let Some(row) = self.search_results.as_ref().and_then(|r| r.results.get(i)) else {
            return;
        };
        let instrument_type = row.type_.clone();
        let Some(sym) = normalize_symbol(&row.ticker) else {
            return;
        };
        self.remember_symbol_kind_from_instrument_type(&sym, &instrument_type);
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
                    let flash = r
                        .as_ref()
                        .ok()
                        .map(|_| crate::app::open_url::NewsUrlFlashHint::Copied);
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
                if self
                    .active_runtime_error
                    .as_ref()
                    .is_some_and(|a| a.source_domain == ErrorSourceDomain::NewsOpenUrl)
                {
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
            7 => {
                self.settings_editing = Some(SettingsEdit::BacktestCapital);
                self.settings_edit_buffer = format!("{}", self.config.backtest.initial_capital);
            }
            8 => {
                self.settings_editing = Some(SettingsEdit::BacktestCommission);
                self.settings_edit_buffer =
                    format!("{}", self.config.backtest.commission_per_trade);
            }
            9 => {
                self.settings_editing = Some(SettingsEdit::BacktestSlippage);
                self.settings_edit_buffer = format!("{}", self.config.backtest.slippage_bps);
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
                    self.settings_inline_error =
                        Some("Refresh rate must be a positive integer.".into());
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
                    if self
                        .active_runtime_error
                        .as_ref()
                        .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
                    {
                        self.active_runtime_error = None;
                    }
                    self.reset_network_poll_clocks();
                    self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
                }
            }
            SettingsEdit::DefaultSymbol => {
                let Some(sym) = normalize_symbol(&self.settings_edit_buffer) else {
                    self.settings_inline_error = Some("Default symbol cannot be empty.".into());
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
                    if self
                        .active_runtime_error
                        .as_ref()
                        .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
                    {
                        self.active_runtime_error = None;
                    }
                    self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
                }
            }
            SettingsEdit::BacktestCapital => {
                let trimmed = self.settings_edit_buffer.trim();
                let Ok(v) = trimmed.parse::<f64>() else {
                    self.settings_inline_error = Some("Initial capital must be a number.".into());
                    return true;
                };
                if v <= 0.0 {
                    self.settings_inline_error = Some("Initial capital must be positive.".into());
                    return true;
                }
                self.config.backtest.initial_capital = v;
                self.settings_save_backtest_row();
            }
            SettingsEdit::BacktestCommission => {
                let trimmed = self.settings_edit_buffer.trim();
                let Ok(v) = trimmed.parse::<f64>() else {
                    self.settings_inline_error = Some("Commission must be a number.".into());
                    return true;
                };
                if v < 0.0 {
                    self.settings_inline_error = Some("Commission cannot be negative.".into());
                    return true;
                }
                self.config.backtest.commission_per_trade = v;
                self.settings_save_backtest_row();
            }
            SettingsEdit::BacktestSlippage => {
                let trimmed = self.settings_edit_buffer.trim();
                let Ok(v) = trimmed.parse::<f64>() else {
                    self.settings_inline_error = Some("Slippage (bps) must be a number.".into());
                    return true;
                };
                if v < 0.0 {
                    self.settings_inline_error = Some("Slippage cannot be negative.".into());
                    return true;
                }
                self.config.backtest.slippage_bps = v;
                self.settings_save_backtest_row();
            }
        }
        self.settings_editing = None;
        self.settings_edit_buffer.clear();
        true
    }

    fn settings_save_backtest_row(&mut self) {
        let _ = self.persist_config_interactive(
            Tab::Settings,
            ErrorSourceDomain::Settings,
            "backtest settings",
        );
        crate::app::backtest_ui::rebuild_backtest_params_cache(self);
        self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
    }

    pub fn settings_try_enter_row(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        match self.settings_row {
            0 | 1 | 7 | 8 | 9 => self.settings_begin_edit(),
            2 => self.settings_toggle_notifications(),
            3 => self.settings_commit_theme_preset(),
            4 => self.settings_toggle_provider(),
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
            if self
                .active_runtime_error
                .as_ref()
                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
            {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
            self.on_theme_preset_committed();
        }
    }

    /// Update-phase hook after a successful Settings theme preset save (Issues #196 / §62, #195 / §61).
    ///
    /// Stock View watchlist, Portfolio, and Alerts build styled widgets per frame from live
    /// [`ResolvedTheme`] and do not need explicit invalidation here. Only Update-phase baked
    /// caches (currently Options [`OptionsDisplayCache`]) are synced.
    fn on_theme_preset_committed(&mut self) {
        crate::app::options::refresh_options_display_for_theme(self);
        // THEME_BAKED_CACHE: register future pre-built styled widget caches here.
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
            if self
                .active_runtime_error
                .as_ref()
                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
            {
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

    /// Clears session symbol-kind metadata after provider change (Issue #160 / §45.1).
    fn clear_symbol_kind_cache(&mut self) {
        self.symbol_kind_cache.clear();
        crate::app::options::clear_options_session(self);
        self.last_options_network_poll = None;
    }

    /// Toggle `yahoo` ↔ `polygon`, clear Kind cache, persist, and refresh quotes (§45.1).
    pub(crate) fn settings_toggle_provider(&mut self) {
        self.settings_inline_error = None;
        let next = match self.config.provider {
            MarketProviderKind::Yahoo => MarketProviderKind::Polygon,
            MarketProviderKind::Polygon => MarketProviderKind::Yahoo,
        };
        if next == MarketProviderKind::Polygon && self.config.effective_api_key().is_empty() {
            self.settings_inline_error = Some(MISSING_API_KEY_FOR_POLYGON_MSG.into());
            return;
        }
        if next == self.config.provider {
            return;
        }
        let previous = self.config.provider;
        self.clear_symbol_kind_cache();
        self.clear_charts_polygon_notice();
        self.config.provider = next;
        if let Err(e) = self.try_save_config_with_session() {
            self.config.provider = previous;
            self.surface_runtime_error(
                Tab::Settings,
                ErrorSourceDomain::Settings,
                AppError::ConfigSave(format!("Failed to save settings: {e}")),
                true,
            );
        } else {
            if self
                .active_runtime_error
                .as_ref()
                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
            {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
            self.reset_network_poll_clocks();
            self.request_immediate_stock_poll();
        }
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
            if self
                .active_runtime_error
                .as_ref()
                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Settings)
            {
                self.active_runtime_error = None;
            }
            self.settings_saved_flash_until = Some(Instant::now() + SETTINGS_SAVED_FLASH);
        }
    }

    fn apply_inflight_recovery(&mut self, kind: InflightRecovery) {
        match kind {
            InflightRecovery::Historical => {
                self.hist_refresh_inflight = false;
                self.hist_inflight_since = None;
            }
            InflightRecovery::News => {
                self.news_refresh_inflight = false;
                self.news_inflight_since = None;
            }
            InflightRecovery::Search => {
                self.search_refresh_inflight = false;
                self.search_inflight_since = None;
            }
            InflightRecovery::Stock => {
                self.stock_refresh_inflight = false;
                self.stock_inflight_since = None;
                // Issue #77 / SPEC §16.3: coalesced refresh must not stick pending when FetchDone send failed.
                if std::mem::take(&mut self.stock_refresh_pending) {
                    self.spawn_stock_fetch_task();
                }
            }
            InflightRecovery::NewsUrlOp => self.news_url_op_inflight = false,
            InflightRecovery::Backtest => {
                self.backtest_inflight = false;
                self.backtest_inflight_since = None;
                self.backtest_draw_cache = None;
            }
            InflightRecovery::Options => {
                self.options_inflight = false;
                self.options_inflight_since = None;
            }
        }
    }

    fn apply_fetch_done(&mut self, msg: FetchDone) {
        match msg {
            FetchDone::Stock {
                generation,
                quotes,
                instrument_types,
                errors,
            } => self.apply_stock_fetch_done(generation, quotes, instrument_types, errors),
            FetchDone::Historical {
                symbol,
                time_range,
                result,
            } => {
                self.hist_refresh_inflight = false;
                self.hist_inflight_since = None;
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
                        self.sync_charts_polygon_notice_from_hist();
                        // Bars may differ on refresh (same symbol/range); drop stale BT report/hint.
                        crate::app::backtest_ui::clear_backtest_session(self);
                        self.historical_data_stamp = self.historical_data_stamp.saturating_add(1);
                        self.invalidate_candle_layout();
                        self.rebuild_chart_indicator_cache();
                        crate::app::backtest_ui::rebuild_backtest_params_cache(self);
                        if matches!(self.last_failed_fetch, LastFailedFetch::Historical) {
                            self.last_failed_fetch = LastFailedFetch::None;
                        }
                        if self
                            .active_runtime_error
                            .as_ref()
                            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Charts)
                        {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                    }
                    Err(err) => {
                        self.clear_charts_polygon_notice();
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
                self.news_inflight_since = None;
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
                        if self
                            .active_runtime_error
                            .as_ref()
                            .is_some_and(|a| a.source_domain == ErrorSourceDomain::News)
                        {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                        if n == 0 {
                            self.news_list_state.select(None);
                        } else {
                            let i = self.news_list_state.selected().unwrap_or(0).min(n - 1);
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
                self.search_inflight_since = None;
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
                        self.cache_symbol_kinds_from_search_results();
                        if matches!(self.last_failed_fetch, LastFailedFetch::Search { .. }) {
                            self.last_failed_fetch = LastFailedFetch::None;
                        }
                        if self
                            .active_runtime_error
                            .as_ref()
                            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Search)
                        {
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
                            let i = self.search_table_state.selected().unwrap_or(0).min(n - 1);
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
            FetchDone::Options {
                symbol,
                expiration_ts,
                result,
                extra_slices,
            } => {
                self.options_inflight = false;
                self.options_inflight_since = None;
                if symbol != self.symbol {
                    return;
                }
                self.last_options_network_poll = Some(Instant::now());
                match result {
                    Ok(chain) => {
                        if chain.expirations.is_empty() {
                            crate::app::options::clear_options_session(self);
                            self.options_no_listed = true;
                        } else {
                            self.options_no_listed = false;
                            if expiration_ts.is_none() {
                                self.options_slices_by_ts.clear();
                            }
                            crate::app::options::merge_options_inline_slices(
                                &mut self.options_slices_by_ts,
                                &chain,
                                &extra_slices,
                            );
                            if self.config.provider == MarketProviderKind::Polygon
                                && !chain.expirations.is_empty()
                            {
                                let wire = resolve_provider_symbol(
                                    MarketProviderKind::Polygon,
                                    &self.symbol,
                                );
                                self.options_polygon_expirations_cache =
                                    Some((wire, chain.expirations.clone()));
                            }
                            let spot = self.get_current_price(&self.symbol);
                            self.options_selected_strike =
                                Some(crate::app::options::default_selected_strike(&chain, spot));
                            self.options_chain = Some(chain);
                            crate::app::options::rebuild_options_display_cache(self);
                            if self
                                .active_runtime_error
                                .as_ref()
                                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Options)
                            {
                                self.clear_active_runtime_unless_alerts_save();
                            }
                        }
                    }
                    Err(err) => {
                        let preserve_chain =
                            expiration_ts.is_some() && self.options_chain_matches_symbol();
                        if preserve_chain {
                            self.surface_runtime_error(
                                Tab::Options,
                                ErrorSourceDomain::Options,
                                AppError::Provider(err),
                                true,
                            );
                        } else {
                            let no_opts = crate::api::error::provider_error_is_no_options(&err);
                            crate::app::options::clear_options_session(self);
                            if no_opts {
                                self.options_no_listed = true;
                            } else {
                                self.options_no_listed = false;
                                self.surface_runtime_error(
                                    Tab::Options,
                                    ErrorSourceDomain::Options,
                                    AppError::Provider(err),
                                    true,
                                );
                            }
                        }
                    }
                }
            }
            FetchDone::Backtest { result } => {
                self.backtest_inflight = false;
                self.backtest_inflight_since = None;
                match result {
                    Ok(report) => {
                        let n = report.trades.len();
                        self.backtest_report = Some(report);
                        crate::app::backtest_ui::rebuild_backtest_draw_cache(self);
                        if self.backtest_trade_list_state.selected().is_none() && n > 0 {
                            self.backtest_trade_list_state.select(Some(0));
                        }
                        if self
                            .active_runtime_error
                            .as_ref()
                            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Backtest)
                        {
                            self.clear_active_runtime_unless_alerts_save();
                        }
                    }
                    Err(err) => {
                        crate::app::backtest_ui::clear_backtest_session(self);
                        self.surface_runtime_error(
                            Tab::Backtest,
                            ErrorSourceDomain::Backtest,
                            AppError::Internal(err.to_string()),
                            true,
                        );
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
        let event_handle = spawn_event_thread(event_tx);

        self.request_immediate_stock_poll();

        let run_result = loop {
            draw(terminal, self)?;

            if self.should_quit {
                self.persist_config_on_shutdown();
                break Ok(());
            }

            tokio::select! {
                ev = event_rx.recv() => {
                    match ev {
                        Some(Event::Input(input)) => {
                            handle_event(self, input);
                            if self.should_fetch_ticker {
                                self.should_fetch_ticker = false;
                                self.commit_stock_symbol_from_input();
                            }
                        }
                        Some(Event::Tick) => self.on_background_tick(),
                        None => {
                            // Event sender dropped (abnormal); best-effort persist like quit path.
                            self.persist_config_on_shutdown();
                            break Ok(());
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
        };

        self.fetch_done_tx = None;
        self.inflight_recovery_tx = None;
        self.url_op_tx = None;
        // Drop the receiver so the event thread's `send` fails and the loop exits (Issue #108).
        drop(event_rx);
        join_event_thread(event_handle);
        run_result
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Stores provider-derived kind for a normalized symbol (Issue #158 / §44.2).
    pub fn remember_symbol_kind(&mut self, normalized: &str, kind: SymbolKind) {
        if kind != SymbolKind::Unknown {
            self.symbol_kind_cache.insert(normalized.to_string(), kind);
        }
    }

    /// Maps Yahoo **`quoteType`** / Search **`type_`** into the session cache.
    pub fn remember_symbol_kind_from_instrument_type(
        &mut self,
        normalized: &str,
        instrument_type: &str,
    ) {
        let kind = classify_from_instrument_type(instrument_type);
        self.remember_symbol_kind(normalized, kind);
    }

    /// Kind for UI: session cache, then string heuristics (§44.2).
    pub fn symbol_kind_for_display(&self, sym: &str) -> SymbolKind {
        let Some(n) = normalize_symbol(sym) else {
            return SymbolKind::Unknown;
        };
        if let Some(&k) = self.symbol_kind_cache.get(&n) {
            return k;
        }
        classify_symbol_with_hint(&n, None)
    }

    fn cache_symbol_kinds_from_search_results(&mut self) {
        let rows: Vec<(String, String)> = self
            .search_results
            .as_ref()
            .map(|r| {
                r.results
                    .iter()
                    .filter_map(|row| normalize_symbol(&row.ticker).map(|n| (n, row.type_.clone())))
                    .collect()
            })
            .unwrap_or_default();
        for (n, t) in rows {
            self.remember_symbol_kind_from_instrument_type(&n, &t);
        }
    }

    /// Aligns watchlist table selection with [`Self::symbol`] (compares via [`symbols_equivalent`]).
    ///
    /// When `symbol` is not on the watchlist (or hidden by the active filter), clears the row
    /// highlight and **does not** change `symbol` (detail pane keeps a typed ticker off-list).
    pub fn sync_watchlist_selection_to_symbol(&mut self) {
        let f = self.watchlist_filter_indices();
        if f.is_empty() {
            self.watchlist_state.select(None);
            return;
        }
        let active = self.symbol.as_str();
        if let Some(full_idx) = self
            .watchlist
            .iter()
            .position(|s| symbols_equivalent(s, active))
        {
            if let Some(sel) = f.iter().position(|&i| i == full_idx) {
                self.watchlist_state.select(Some(sel));
                return;
            }
        }
        self.watchlist_state.select(None);
    }

    /// Stock View **Enter**: normalize typed symbol, sync watchlist highlight, poll immediately.
    ///
    /// Replaces the pre–§43.13 path that called [`sync_watchlist_selection_to_symbol`] alone and
    /// could overwrite a typed off-list ticker with the first watchlist row.
    pub fn commit_stock_symbol_from_input(&mut self) {
        let Some(sym) = normalize_symbol(&self.symbol) else {
            return;
        };
        let symbol_changed = !symbols_equivalent(&self.symbol, &sym);
        self.symbol = sym;
        self.sync_watchlist_selection_to_symbol();
        if symbol_changed {
            self.on_active_symbol_changed_for_charts();
        }
        self.notify_symbol_changed_for_news();
        self.request_immediate_stock_poll();
        self.persist_session_to_disk();
    }

    pub fn add_current_to_watchlist(&mut self) {
        let prev_effective = self.symbol.clone();
        let Some(sym) = normalize_symbol(&self.symbol) else {
            return;
        };
        if self.watchlist.iter().any(|s| symbols_equivalent(s, &sym)) {
            return;
        }
        let same_ticker_case_only = symbols_equivalent(&prev_effective, &sym);
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
        } else if self
            .active_runtime_error
            .as_ref()
            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Portfolio)
        {
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
        self.watchlist_quotes
            .retain(|k, _| self.watchlist.iter().any(|w| symbols_equivalent(w, k)));
        self.symbol_kind_cache
            .retain(|k, _| self.watchlist.iter().any(|w| symbols_equivalent(w, k)));
        self.config.watchlist = self.watchlist.clone();
        if let Err(e) = self.try_save_config_with_session() {
            self.surface_runtime_error(
                Tab::StockView,
                ErrorSourceDomain::Portfolio,
                AppError::ConfigSave(format!("Failed to save watchlist: {e}")),
                true,
            );
        } else if self
            .active_runtime_error
            .as_ref()
            .is_some_and(|a| a.source_domain == ErrorSourceDomain::Portfolio)
        {
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

        self.ticker_data = self.watchlist_quotes.get(&self.symbol).cloned();
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
            None => self.watchlist_state.select(Some(f.len().saturating_sub(1))),
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
            self.chart_indicator_cache = None;
            self.invalidate_candle_layout();
            self.chart_viewport = ChartViewport::default();
            self.clear_charts_polygon_notice();
            crate::app::backtest_ui::clear_backtest_session(self);
            crate::app::backtest_ui::rebuild_backtest_params_cache(self);
            if self
                .active_runtime_error
                .as_ref()
                .is_some_and(|a| a.source_domain == ErrorSourceDomain::Charts)
            {
                self.active_runtime_error = None;
            }
        }
        self.request_immediate_charts_poll();
        if !changed {
            self.charts_reset_viewport();
        }
        self.persist_session_to_disk();
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
        self.invalidate_candle_layout();
    }

    pub fn charts_zoom_out(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        viewport_zoom_out(&mut self.chart_viewport, h.results.len());
        self.invalidate_candle_layout();
    }

    pub fn charts_pan_left(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        crate::app::charts::viewport_pan_left(&mut self.chart_viewport, h.results.len());
        self.invalidate_candle_layout();
    }

    pub fn charts_pan_right(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        crate::app::charts::viewport_pan_right(&mut self.chart_viewport, h.results.len());
        self.invalidate_candle_layout();
    }

    pub fn charts_reset_viewport(&mut self) {
        let Some(h) = self.historical_data.as_ref() else {
            return;
        };
        self.chart_viewport = ChartViewport::full(h.results.len());
        self.invalidate_candle_layout();
    }

    pub fn charts_toggle_mode(&mut self) {
        self.chart_mode = self.chart_mode.toggle();
        self.persist_session_to_disk();
    }

    /// Issue #199 / §64 — drop cached candle layout (called from Update phase).
    pub(crate) fn invalidate_candle_layout(&mut self) {
        self.chart_candle_layout = None;
    }

    /// Issue #199 / §64 — prepare candle layout cache for draw (called from UI draw path).
    ///
    /// Rebuilds cache only if key changed (viewport, bars, time_range, area, series_stamp).
    pub(crate) fn prepare_charts_draw_cache(&mut self, price_area: ratatui::layout::Rect) {
        let Some(historical_data) = &self.historical_data else {
            return;
        };
        let slice =
            crate::app::charts::visible_slice(&historical_data.results, &self.chart_viewport);
        if slice.is_empty() {
            return;
        }

        let key = crate::app::charts::ChartCandleLayoutKey {
            area: price_area,
            viewport: self.chart_viewport,
            bars_len: slice.len(),
            time_range: self.time_range,
            series_stamp: self.historical_data_stamp,
        };

        if let Some(cache) = &self.chart_candle_layout {
            if cache.key == key {
                return;
            }
        }

        #[cfg(test)]
        CANDLE_LAYOUT_BUILD_COUNTER.with(|c| c.set(c.get().saturating_add(1)));

        let layouts = crate::app::charts::layout_candles(price_area, slice, self.time_range);
        self.chart_candle_layout = Some(ChartCandleLayoutCache { key, layouts });
    }

    /// Toggle SMA(20) overlay on the Charts line chart (Issue #21).
    pub fn charts_toggle_sma_20(&mut self) {
        self.chart_indicators.sma_20 = !self.chart_indicators.sma_20;
        self.sync_chart_indicator_cache_after_toggle();
    }

    /// Toggle EMA(20) overlay on the Charts line chart (Issue #21).
    pub fn charts_toggle_ema_20(&mut self) {
        self.chart_indicators.ema_20 = !self.chart_indicators.ema_20;
        self.sync_chart_indicator_cache_after_toggle();
    }

    /// Toggle RSI(14) sub-pane on the Charts tab (Issue #21).
    pub fn charts_toggle_rsi_14(&mut self) {
        self.chart_indicators.rsi_14 = !self.chart_indicators.rsi_14;
        self.sync_chart_indicator_cache_after_toggle();
    }

    /// Toggle MACD(12/26/9) sub-pane on the Charts tab (Issue #21).
    pub fn charts_toggle_macd(&mut self) {
        self.chart_indicators.macd = !self.chart_indicators.macd;
        self.sync_chart_indicator_cache_after_toggle();
    }

    /// Runs backtest off the UI thread (Issue #25 / §47.3).
    pub fn request_backtest_run(&mut self) {
        if self.backtest_inflight {
            return;
        }
        let Some(hist) = self.historical_data.as_ref() else {
            self.surface_runtime_error(
                Tab::Backtest,
                ErrorSourceDomain::Backtest,
                AppError::Internal("Load chart data first (Charts tab, Y1 recommended).".into()),
                true,
            );
            return;
        };
        if hist.results.is_empty() {
            self.surface_runtime_error(
                Tab::Backtest,
                ErrorSourceDomain::Backtest,
                AppError::Internal("Historical series is empty.".into()),
                true,
            );
            return;
        };
        if let Err(e) = crate::backtest::verify_historical_symbol(hist, &self.symbol) {
            self.surface_runtime_error(
                Tab::Backtest,
                ErrorSourceDomain::Backtest,
                AppError::Internal(e.to_string()),
                true,
            );
            return;
        }
        let Some(fetch_tx) = self.fetch_done_tx.clone() else {
            return;
        };
        let recovery_tx = self.inflight_recovery_tx.clone();
        let bars = hist.results.clone();
        let symbol = self.symbol.clone();
        let sim = self.config.backtest.clone();
        let strategy = self.config.backtest_strategy.clone();
        self.backtest_inflight = true;
        self.backtest_inflight_since = Some(Instant::now());
        tokio::task::spawn_blocking(move || {
            let result = match std::panic::catch_unwind(AssertUnwindSafe(|| {
                crate::backtest::run_backtest(&symbol, &bars, &sim, &strategy)
            })) {
                Ok(r) => r,
                Err(payload) => {
                    #[cfg(debug_assertions)]
                    log_quote_batch_panic(&*payload);
                    #[cfg(not(debug_assertions))]
                    drop(payload);
                    Err(crate::backtest::BacktestError::TaskPanicked)
                }
            };
            crate::app::fetch_delivery::deliver_fetch_done(
                &fetch_tx,
                recovery_tx.as_ref(),
                FetchDone::Backtest { result },
                InflightRecovery::Backtest,
            );
        });
    }

    fn options_chain_matches_symbol(&self) -> bool {
        self.options_chain
            .as_ref()
            .is_some_and(|c| symbols_equivalent(&c.slice.underlying, self.symbol.as_str()))
    }

    /// Fetches options chain off the UI thread (Issue #22 / §48.3).
    pub fn request_options_fetch(&mut self, expiration_ts: Option<u64>) {
        if self.options_inflight || self.symbol.is_empty() {
            return;
        }
        let Some(fetch_tx) = self.fetch_done_tx.clone() else {
            return;
        };
        let sym = self.symbol.clone();
        let cfg = self.config.clone();
        let recovery_tx = self.inflight_recovery_tx.clone();
        let polygon_cached_expirations = if cfg.provider == MarketProviderKind::Polygon {
            if expiration_ts.is_none() {
                self.options_polygon_expirations_cache = None;
                None
            } else {
                let wire = resolve_provider_symbol(MarketProviderKind::Polygon, &sym);
                self.options_polygon_expirations_cache
                    .as_ref()
                    .filter(|(w, ex)| w == &wire && !ex.is_empty())
                    .map(|(_, ex)| ex.clone())
            }
        } else {
            None
        };
        self.options_inflight = true;
        self.options_inflight_since = Some(Instant::now());
        self.options_no_listed = false;
        tokio::spawn(async move {
            let (result, extra_slices) = match cfg.provider {
                MarketProviderKind::Yahoo => {
                    match crate::api::yahoo_options::yahoo_options_chain_with_slices(
                        &sym,
                        expiration_ts,
                    )
                    .await
                    {
                        Ok(parsed) => (Ok(parsed.chain), parsed.slices_by_ts),
                        Err(e) => (Err(e), std::collections::HashMap::new()),
                    }
                }
                MarketProviderKind::Polygon => {
                    let cached = polygon_cached_expirations.as_deref();
                    match crate::api::polygon_options::polygon_options_chain_with_slices(
                        &sym,
                        expiration_ts,
                        &cfg,
                        cached,
                    )
                    .await
                    {
                        Ok(parsed) => (Ok(parsed.chain), parsed.slices_by_ts),
                        Err(e) => (Err(e), std::collections::HashMap::new()),
                    }
                }
            };
            crate::app::fetch_delivery::deliver_fetch_done(
                &fetch_tx,
                recovery_tx.as_ref(),
                FetchDone::Options {
                    symbol: sym,
                    expiration_ts,
                    result,
                    extra_slices,
                },
                InflightRecovery::Options,
            );
        });
    }

    /// Select expiration from session cache; network only on cache miss (Issue #168 / §49.2).
    pub fn options_select_expiration(&mut self, ts: u64) {
        if self.options_inflight {
            return;
        }
        let Some(chain) = self.options_chain.as_ref() else {
            self.request_options_fetch(Some(ts));
            return;
        };
        if !chain.expirations.iter().any(|e| e.ts == ts) {
            return;
        }
        if let Some(slice) = self.options_slices_by_ts.get(&ts).cloned() {
            if std::env::var("STOCKTERM_DEBUG_YAHOO_OPTIONS").as_deref() == Ok("1") {
                tracing::info!(
                    target: "stockterm::yahoo_options",
                    ts,
                    slices = self.options_slices_by_ts.len(),
                    "options expiration cache hit"
                );
            }
            if std::env::var("STOCKTERM_DEBUG_POLYGON_OPTIONS").as_deref() == Ok("1") {
                tracing::info!(
                    target: "stockterm::polygon_options",
                    ts,
                    slices = self.options_slices_by_ts.len(),
                    "options expiration slice cache hit"
                );
            }
            if let Some(c) = self.options_chain.as_mut() {
                c.selected_expiration_ts = ts;
                c.slice = slice;
            }
            let spot = self.get_current_price(&self.symbol);
            if let Some(ref c) = self.options_chain {
                self.options_selected_strike =
                    Some(crate::app::options::default_selected_strike(c, spot));
            }
            crate::app::options::rebuild_options_display_cache(self);
            return;
        }
        if std::env::var("STOCKTERM_DEBUG_YAHOO_OPTIONS").as_deref() == Ok("1") {
            tracing::info!(
                target: "stockterm::yahoo_options",
                ts,
                "options expiration cache miss spawning fetch"
            );
        }
        if std::env::var("STOCKTERM_DEBUG_POLYGON_OPTIONS").as_deref() == Ok("1") {
            tracing::info!(
                target: "stockterm::polygon_options",
                ts,
                polygon_expirations_cached = self.options_polygon_expirations_cache.is_some(),
                "options expiration slice cache miss spawning fetch"
            );
        }
        self.request_options_fetch(Some(ts));
    }

    /// Auto-fetch when Options tab is active and chain is missing/stale (§48.3).
    pub fn try_spawn_options_fetch(&mut self) {
        if self.options_inflight || self.symbol.is_empty() {
            return;
        }
        if self.config.provider != MarketProviderKind::Yahoo {
            return;
        }
        if self.options_no_listed {
            return;
        }
        if self.options_chain_matches_symbol() {
            return;
        }
        let due = self
            .last_options_network_poll
            .map(|t| t.elapsed() >= self.data_poll_interval())
            .unwrap_or(true);
        if !due {
            return;
        }
        let expiration_ts = self
            .options_chain
            .as_ref()
            .map(|c| c.selected_expiration_ts);
        self.request_options_fetch(expiration_ts);
    }

    /// Cycle to previous expiration (Issue #22; cache-aware — Issue #168).
    pub fn options_expiration_prev(&mut self) {
        let Some(ts) = (|| {
            let chain = self.options_chain.as_ref()?;
            let idx = chain
                .expirations
                .iter()
                .position(|e| e.ts == chain.selected_expiration_ts)
                .unwrap_or(0);
            let new_idx = idx.saturating_sub(1);
            if new_idx == idx {
                return None;
            }
            Some(chain.expirations[new_idx].ts)
        })() else {
            if self.options_chain.is_none() {
                self.request_options_fetch(None);
            }
            return;
        };
        self.options_select_expiration(ts);
    }

    /// Cycle to next expiration (Issue #22; cache-aware — Issue #168).
    pub fn options_expiration_next(&mut self) {
        let Some(ts) = (|| {
            let chain = self.options_chain.as_ref()?;
            let idx = chain
                .expirations
                .iter()
                .position(|e| e.ts == chain.selected_expiration_ts)
                .unwrap_or(0);
            let new_idx = (idx + 1).min(chain.expirations.len().saturating_sub(1));
            if new_idx == idx {
                return None;
            }
            Some(chain.expirations[new_idx].ts)
        })() else {
            if self.options_chain.is_none() {
                self.request_options_fetch(None);
            }
            return;
        };
        self.options_select_expiration(ts);
    }

    /// Move shared strike highlight (j/k) along the canonical strike list (calls + puts).
    pub fn options_strike_scroll(&mut self, down: bool) {
        let Some(chain) = self.options_chain.as_ref() else {
            return;
        };
        let strikes = crate::app::options::canonical_strikes(chain);
        if strikes.is_empty() {
            return;
        }
        let cur = self.options_selected_strike.unwrap_or(strikes[0]);
        let idx = strikes
            .iter()
            .position(|&s| (s - cur).abs() < 1e-6)
            .unwrap_or(0);
        let next_idx = if down {
            (idx + 1).min(strikes.len() - 1)
        } else {
            idx.saturating_sub(1)
        };
        self.options_selected_strike = Some(strikes[next_idx]);
        crate::app::options::sync_options_table_states(self);
    }

    /// Toggle optional Greeks columns (session-only).
    pub fn options_toggle_greeks(&mut self) {
        self.options_show_greeks = !self.options_show_greeks;
        crate::app::options::rebuild_options_display_cache(self);
    }

    /// Compact status suffix for Options tab (§48.4).
    pub fn options_status_suffix(&self) -> Option<String> {
        if self.options_inflight {
            return Some("Loading options…".into());
        }
        let chain = self.options_chain.as_ref()?;
        let strikes = chain.slice.calls.len().max(chain.slice.puts.len());
        let g = if self.options_show_greeks {
            "on"
        } else {
            "off"
        };
        Some(format!(
            "OPT: {} │ {strikes} strikes │ g {g}",
            chain.slice.expiration.label
        ))
    }

    /// Cycles SMA ↔ RSI strategy and persists config (Issue #25).
    ///
    /// Does not auto-rerun; operator presses **Enter** / **`r`** on the Backtest tab.
    pub fn backtest_cycle_strategy(&mut self) {
        use crate::models::backtest::BacktestStrategyKind;
        let previous = self.config.backtest_strategy.kind;
        self.config.backtest_strategy.kind = match previous {
            BacktestStrategyKind::SmaCrossover => BacktestStrategyKind::RsiMeanReversion,
            BacktestStrategyKind::RsiMeanReversion => BacktestStrategyKind::SmaCrossover,
        };
        if !self.persist_config_interactive(
            Tab::Backtest,
            ErrorSourceDomain::Backtest,
            "backtest settings",
        ) {
            self.config.backtest_strategy.kind = previous;
        }
        crate::app::backtest_ui::rebuild_backtest_params_cache(self);
    }

    /// Scroll trade list on Backtest tab.
    pub fn backtest_trade_scroll(&mut self, down: bool) {
        let n = self
            .backtest_report
            .as_ref()
            .map(|r| r.trades.len())
            .unwrap_or(0);
        if n == 0 {
            return;
        }
        let cur = self.backtest_trade_list_state.selected().unwrap_or(0);
        let next = if down {
            (cur + 1).min(n - 1)
        } else {
            cur.saturating_sub(1)
        };
        self.backtest_trade_list_state.select(Some(next));
    }

    /// Writes JSON + CSV export under `~/.stockterm/` (Issue #25 / §47.6).
    ///
    /// Runs synchronously on the UI thread; intended for small report sizes.
    pub fn backtest_export_to_disk(&mut self) -> Result<String, String> {
        use crate::models::backtest::BacktestExportBundle;
        use std::fs;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        let report = self
            .backtest_report
            .as_ref()
            .ok_or_else(|| "Run a backtest first (Enter or r).".to_string())?;
        let home = dirs::home_dir().ok_or_else(|| "home directory not found".to_string())?;
        let dir = home.join(".stockterm");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let sym = report.summary.symbol.replace('/', "_");
        let stem = format!("backtest_{sym}_{ts}");
        let json_path = dir.join(format!("{stem}.json"));
        let csv_path = dir.join(format!("{stem}.csv"));

        let bundle = BacktestExportBundle {
            report: report.clone(),
            sim: self.config.backtest.clone(),
            strategy: self.config.backtest_strategy.clone(),
        };
        let json = serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?;
        fs::write(&json_path, json).map_err(|e| e.to_string())?;

        let mut csv =
            std::io::BufWriter::new(fs::File::create(&csv_path).map_err(|e| e.to_string())?);
        writeln!(
            csv,
            "entry_ts,exit_ts,side,entry_price,exit_price,shares,pnl"
        )
        .map_err(|e| e.to_string())?;
        for t in &report.trades {
            writeln!(
                csv,
                "{},{},{},{},{},{},{}",
                t.entry_ts, t.exit_ts, t.side, t.entry_price, t.exit_price, t.shares, t.pnl
            )
            .map_err(|e| e.to_string())?;
        }
        writeln!(csv, "# symbol,{}", report.summary.symbol).map_err(|e| e.to_string())?;
        writeln!(csv, "# total_pnl,{}", report.summary.total_pnl).map_err(|e| e.to_string())?;
        writeln!(
            csv,
            "# max_drawdown_pct,{}",
            report.summary.max_drawdown_pct
        )
        .map_err(|e| e.to_string())?;
        csv.flush().map_err(|e| e.to_string())?;

        let msg = format!(
            "Exported to {} and {}",
            csv_path.display(),
            json_path.display()
        );
        self.backtest_flash = Some((msg.clone(), Instant::now() + Duration::from_secs(5)));
        Ok(msg)
    }

    /// Status hint for Backtest tab (export flash or last run summary).
    pub fn backtest_status_hint(&self) -> Option<String> {
        if let Some((msg, until)) = &self.backtest_flash {
            if Instant::now() < *until {
                return Some(msg.clone());
            }
        }
        if self.backtest_inflight {
            return Some("Running backtest…".into());
        }
        self.backtest_report.as_ref().map(|r| {
            format!(
                "BT: {:+.1}% │ {} trades",
                r.summary.total_return_pct, r.summary.trade_count
            )
        })
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
            Tab::Settings => Tab::Backtest,
            Tab::Backtest => Tab::Options,
            Tab::Options => Tab::StockView,
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
            Tab::StockView => Tab::Options,
            Tab::Portfolio => Tab::StockView,
            Tab::Alerts => Tab::Portfolio,
            Tab::Search => Tab::Alerts,
            Tab::News => Tab::Search,
            Tab::Charts => Tab::News,
            Tab::Settings => Tab::Charts,
            Tab::Backtest => Tab::Settings,
            Tab::Options => Tab::Backtest,
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
            .find(|i| symbols_equivalent(&i.symbol, &sym))
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
                            .position(|&i| symbols_equivalent(&self.portfolio[i].symbol, &sym))
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

    /// Overwrites shares and avg cost for the holding at `index` and persists config (Issue #182 / §55).
    ///
    /// # Returns
    ///
    /// - `true` if the row existed and [`Self::try_save_config_with_session`] succeeded.
    /// - `false` if index out of range or save failed (save failure sets runtime error).
    pub fn update_portfolio_holding(
        &mut self,
        index: usize,
        shares: f64,
        purchase_price: f64,
    ) -> bool {
        if index >= self.portfolio.len() {
            return false;
        }

        let backup = self.portfolio.clone();
        self.portfolio[index].shares = shares;
        self.portfolio[index].purchase_price = purchase_price;
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

#[cfg(test)]
mod tests {
    use super::{
        data_poll_interval_secs, search_result_matches_current, App, ChartDisplayMode, FetchDone,
    };
    use crate::app::app_error::{push_error_log, ErrorLogEntry, UiErrorCategory, ERROR_LOG_CAP};
    use crate::app::Tab;
    use crate::config::Config;
    use crate::models::time_range::TimeRange;
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

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
        app.apply_stock_fetch_done(1, HashMap::new(), HashMap::new(), errors);
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
            AppError::Internal(format!("{ALERTS_SAVE_ERROR_PREFIX} disk · first-batch")),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Stock,
        ));

        let errors = vec![("MSFT".into(), ProviderError::ApiMessage("second".into()))];
        app.apply_stock_fetch_done(1, HashMap::new(), HashMap::new(), errors);
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
        assert_eq!(Tab::from_config_str("backtest"), Some(Tab::Backtest));
        assert_eq!(Tab::from_config_str("options"), Some(Tab::Options));
        assert_eq!(Tab::Options.as_config_str(), "options");
        assert!(Tab::from_config_str("nope").is_none());
    }

    #[test]
    fn sync_session_fields_writes_chart_prefs_issue_180() {
        let mut app = App::new();
        app.time_range = TimeRange::D1;
        app.chart_mode = ChartDisplayMode::Candlestick;
        app.sync_session_fields_into_config();
        assert_eq!(app.config.last_time_range.as_deref(), Some("d1"));
        assert_eq!(app.config.last_chart_mode.as_deref(), Some("candles"));
    }

    #[test]
    fn restore_chart_prefs_from_config_strings_issue_180() {
        let mut config = Config {
            last_time_range: Some("y1".into()),
            last_chart_mode: Some("candles".into()),
            ..Default::default()
        };
        let tr = config
            .last_time_range
            .as_deref()
            .and_then(TimeRange::from_config_str)
            .unwrap_or_default();
        let mode = config
            .last_chart_mode
            .as_deref()
            .and_then(ChartDisplayMode::from_config_str)
            .unwrap_or_default();
        assert_eq!(tr, TimeRange::Y1);
        assert_eq!(mode, ChartDisplayMode::Candlestick);

        config.last_time_range = Some("bogus".into());
        config.last_chart_mode = Some("invalid".into());
        assert_eq!(
            config
                .last_time_range
                .as_deref()
                .and_then(TimeRange::from_config_str)
                .unwrap_or_default(),
            TimeRange::M1
        );
        assert_eq!(
            config
                .last_chart_mode
                .as_deref()
                .and_then(ChartDisplayMode::from_config_str)
                .unwrap_or_default(),
            ChartDisplayMode::Line
        );
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
            AppError::Internal(format!("{ALERTS_SAVE_ERROR_PREFIX} simulated · API: bad")),
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
    fn sync_watchlist_selection_keeps_typed_symbol_not_in_watchlist() {
        let mut app = App::new();
        app.watchlist = vec!["AAPL".into(), "MSFT".into()];
        app.symbol = "BTC-USD".to_string();
        app.sync_watchlist_selection_to_symbol();
        assert_eq!(app.symbol, "BTC-USD");
        assert!(app.watchlist_state.selected().is_none());
    }

    #[test]
    fn commit_stock_symbol_from_input_normalizes_and_keeps_off_watchlist() {
        let mut app = App::new();
        app.watchlist = vec!["AAPL".into()];
        app.symbol = "eth-usd".to_string();
        app.commit_stock_symbol_from_input();
        assert_eq!(app.symbol, "ETH-USD");
        assert!(app.watchlist_state.selected().is_none());
    }

    #[test]
    fn commit_stock_symbol_from_input_case_only_keeps_historical_data() {
        use crate::models::historical::{HistoricalData, HistoricalResponse};

        let mut app = App::new();
        app.symbol = "aapl".to_string();
        app.historical_data = Some(HistoricalResponse {
            ticker: "AAPL".into(),
            results: vec![HistoricalData {
                o: 1.0,
                h: 2.0,
                l: 0.5,
                c: 1.5,
                v: 100.0,
                t: 1,
                vw: 1.0,
                n: None,
            }],
            ..Default::default()
        });
        let stamp_before = app.historical_data_stamp;
        app.commit_stock_symbol_from_input();
        assert_eq!(app.symbol, "AAPL");
        assert!(app.historical_data.is_some());
        assert_eq!(app.historical_data_stamp, stamp_before);
    }

    #[test]
    fn sync_watchlist_selection_matches_equivalent_casing() {
        let mut app = App::new();
        app.watchlist = vec!["AAPL".into()];
        app.symbol = "aapl".to_string();
        app.sync_watchlist_selection_to_symbol();
        assert_eq!(app.watchlist_state.selected(), Some(0));
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
        assert_eq!(app.error_message().as_deref(), Some("[int] quote failed"));
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
        assert!(app
            .error_message()
            .is_some_and(|m| m.contains(ALERTS_SAVE_ERROR_PREFIX)));
    }

    #[test]
    fn collect_symbols_for_quote_fetch_normalizes_watchlist_keys() {
        let mut app = App::new();
        app.watchlist = vec!["btc - usd".into(), "AAPL".into()];
        let syms = app.collect_symbols_for_quote_fetch();
        assert!(syms.contains(&"BTC-USD".to_string()));
        assert!(syms.contains(&"AAPL".to_string()));
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

    /// Issue #192 / SPEC §66 — Backtest strategy toggle surfaces config I/O failure.
    #[cfg(unix)]
    #[test]
    fn backtest_cycle_strategy_surfaces_config_save_error() {
        use crate::app::app_error::{AppError, ErrorSourceDomain};
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;
        use std::sync::Mutex;

        static HOME_TEST_LOCK: Mutex<()> = Mutex::new(());

        struct HomeGuard {
            prev: Option<String>,
        }

        impl HomeGuard {
            fn set(home: &str) -> Self {
                let prev = std::env::var("HOME").ok();
                // SAFETY: held under HOME_TEST_LOCK for the test duration.
                unsafe { std::env::set_var("HOME", home) };
                Self { prev }
            }
        }

        impl Drop for HomeGuard {
            fn drop(&mut self) {
                match &self.prev {
                    Some(v) => unsafe { std::env::set_var("HOME", v) },
                    None => unsafe { std::env::remove_var("HOME") },
                }
            }
        }

        let _lock = HOME_TEST_LOCK.lock().expect("home test lock");
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/_stockterm_backtest_cfg_ro_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join(".stockterm.json");
        fs::write(&path, "{}").expect("write config");
        let mut perms = fs::metadata(&path).expect("meta").permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&path, perms).expect("chmod ro");
        let _home = HomeGuard::set(dir.to_str().expect("utf8 home"));

        let mut app = App::new();
        app.active_tab = Tab::Backtest;
        app.active_runtime_error = None;
        let kind_before = app.config.backtest_strategy.kind;
        app.backtest_cycle_strategy();

        assert_eq!(
            app.config.backtest_strategy.kind, kind_before,
            "strategy kind must revert when persist fails"
        );
        let err = app
            .active_runtime_error
            .as_ref()
            .expect("expected runtime error");
        assert_eq!(err.source_domain, ErrorSourceDomain::Backtest);
        match &err.error {
            AppError::ConfigSave(msg) => {
                assert!(
                    msg.contains("backtest settings"),
                    "expected backtest context in message: {msg}"
                );
            }
            other => panic!("expected ConfigSave, got {other:?}"),
        }
        assert!(
            !app.error_log.is_empty(),
            "cfg save failure should appear in the error log"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// Issue #192 / §66 — successful cfg save must not clear unrelated Backtest `Internal` errors.
    #[cfg(unix)]
    #[test]
    fn persist_config_interactive_clears_only_config_save_in_domain() {
        use crate::app::app_error::{
            ActiveErrorState, AppError, ErrorPersistence, ErrorSourceDomain,
        };
        use std::fs;
        use std::path::PathBuf;
        use std::sync::Mutex;
        use std::time::Instant;

        static HOME_TEST_LOCK: Mutex<()> = Mutex::new(());

        struct HomeGuard {
            prev: Option<String>,
        }

        impl HomeGuard {
            fn set(home: &str) -> Self {
                let prev = std::env::var("HOME").ok();
                // SAFETY: held under HOME_TEST_LOCK for the test duration.
                unsafe { std::env::set_var("HOME", home) };
                Self { prev }
            }
        }

        impl Drop for HomeGuard {
            fn drop(&mut self) {
                match &self.prev {
                    Some(v) => unsafe { std::env::set_var("HOME", v) },
                    None => unsafe { std::env::remove_var("HOME") },
                }
            }
        }

        let _lock = HOME_TEST_LOCK.lock().expect("home test lock");
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/_stockterm_cfg_save_ok_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let _home = HomeGuard::set(dir.to_str().expect("utf8 home"));

        let mut app = App::new();
        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::Internal("Load chart data first.".into()),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Backtest,
        ));
        assert!(app.persist_config_interactive(
            Tab::Backtest,
            ErrorSourceDomain::Backtest,
            "backtest settings",
        ));
        assert!(
            matches!(
                app.active_runtime_error.as_ref().map(|a| &a.error),
                Some(AppError::Internal(_))
            ),
            "Internal backtest error must survive a successful cfg save"
        );

        app.active_runtime_error = Some(ActiveErrorState::new(
            AppError::ConfigSave("Failed to save backtest settings: simulated".into()),
            ErrorPersistence::Sticky,
            Instant::now(),
            ErrorSourceDomain::Backtest,
        ));
        assert!(app.persist_config_interactive(
            Tab::Backtest,
            ErrorSourceDomain::Backtest,
            "backtest settings",
        ));
        assert!(
            app.active_runtime_error.is_none(),
            "prior ConfigSave in the same domain should clear on success"
        );
        let _ = fs::remove_dir_all(&dir);
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
            kind: crate::app::PortfolioDialogKind::Add,
            shares_buffer: "1".into(),
            price_buffer: "1".into(),
            focused: PortfolioAddField::Price,
            inline_error: None,
            commit_armed: false,
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

    /// Issue #78 / SPEC §39.2 — stale watchdog clears stock inflight after channel failures.
    #[test]
    fn recover_stale_inflight_flags_clears_stock_inflight() {
        let _guard = InflightStaleEnvGuard::set_secs(0);
        let mut app = App::new();
        app.stock_refresh_inflight = true;
        app.stock_inflight_since = Some(Instant::now() - Duration::from_millis(10));
        app.recover_stale_inflight_flags();
        assert!(!app.stock_refresh_inflight);
        assert!(app.stock_inflight_since.is_none());
    }

    struct InflightStaleEnvGuard;

    impl InflightStaleEnvGuard {
        fn set_secs(secs: u64) -> Self {
            std::env::set_var("STOCKTERM_INFLIGHT_STALE_SECS", secs.to_string());
            Self
        }
    }

    impl Drop for InflightStaleEnvGuard {
        fn drop(&mut self) {
            std::env::remove_var("STOCKTERM_INFLIGHT_STALE_SECS");
        }
    }

    /// Clears `STOCKTERM_API_KEY` for hermetic provider-toggle tests; restores on drop.
    struct ApiKeyEnvGuard {
        prev: Option<String>,
    }

    impl ApiKeyEnvGuard {
        fn without_env() -> Self {
            let prev = std::env::var("STOCKTERM_API_KEY").ok();
            // SAFETY: test-only; restored in `Drop`.
            unsafe { std::env::remove_var("STOCKTERM_API_KEY") };
            Self { prev }
        }
    }

    impl Drop for ApiKeyEnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => unsafe { std::env::set_var("STOCKTERM_API_KEY", v) },
                None => unsafe { std::env::remove_var("STOCKTERM_API_KEY") },
            }
        }
    }

    /// Issue #160 / SPEC §45.1 — provider toggle clears Yahoo-derived Kind cache.
    #[test]
    fn settings_toggle_provider_clears_symbol_kind_cache() {
        use crate::config::MarketProviderKind;
        use crate::models::symbol::SymbolKind;

        let _env = ApiKeyEnvGuard::without_env();
        let mut app = App::new();
        app.config.provider = MarketProviderKind::Yahoo;
        app.remember_symbol_kind("BTC-USD", SymbolKind::Crypto);
        assert!(app.symbol_kind_cache.contains_key("BTC-USD"));

        app.config.api_key = "test-polygon-key".into();
        app.settings_toggle_provider();
        assert_eq!(app.config.provider, MarketProviderKind::Polygon);
        assert!(app.symbol_kind_cache.is_empty());
    }

    /// Issue #160 / SPEC §45.1 — cannot switch to Polygon without an API key.
    #[test]
    fn settings_toggle_provider_polygon_requires_api_key() {
        use crate::config::MarketProviderKind;

        let _env = ApiKeyEnvGuard::without_env();
        let mut app = App::new();
        app.config.provider = MarketProviderKind::Yahoo;
        app.config.api_key.clear();
        app.settings_toggle_provider();
        assert_eq!(app.config.provider, MarketProviderKind::Yahoo);
        assert!(app.settings_inline_error.is_some());
    }

    /// Issue #160 / audit — stale Yahoo `instrument_types` must not refill cache on Polygon.
    #[test]
    fn apply_stock_fetch_done_skips_instrument_types_when_provider_polygon() {
        use crate::config::MarketProviderKind;
        use crate::models::symbol::SymbolKind;
        use std::collections::HashMap;

        let mut app = App::new();
        app.config.provider = MarketProviderKind::Polygon;
        app.config.api_key = "test-key".into();
        app.stock_fetch_generation = 1;

        let mut instrument_types = HashMap::new();
        instrument_types.insert("BTC".to_string(), "ETF".to_string());

        app.apply_stock_fetch_done(1, HashMap::new(), instrument_types, vec![]);
        assert!(app.symbol_kind_cache.is_empty());
        assert_eq!(app.symbol_kind_for_display("BTC"), SymbolKind::Equity);
    }

    /// Issue #168 / §49.2 — cached expiration switch must not arm inflight fetch.
    #[test]
    fn options_select_expiration_cache_hit() {
        use crate::models::options::{
            Expiration, OptionContract, OptionRight, OptionsChain, OptionsChainSlice,
        };

        let mut app = App::new();
        app.symbol = "AAPL".into();
        let exp1 = Expiration {
            ts: 100,
            label: "2026-06-20".into(),
        };
        let exp2 = Expiration {
            ts: 200,
            label: "2026-06-27".into(),
        };
        let slice1 = OptionsChainSlice {
            underlying: "AAPL".into(),
            expiration: exp1.clone(),
            calls: vec![OptionContract {
                symbol: "C1".into(),
                strike: 100.0,
                right: OptionRight::Call,
                expiration_ts: 100,
                bid: None,
                ask: None,
                last: None,
                volume: None,
                open_interest: None,
                implied_volatility: None,
                greeks: None,
            }],
            puts: vec![],
        };
        let slice2 = OptionsChainSlice {
            underlying: "AAPL".into(),
            expiration: exp2.clone(),
            calls: vec![OptionContract {
                symbol: "C2".into(),
                strike: 200.0,
                right: OptionRight::Call,
                expiration_ts: 200,
                bid: None,
                ask: None,
                last: None,
                volume: None,
                open_interest: None,
                implied_volatility: None,
                greeks: None,
            }],
            puts: vec![],
        };
        app.options_slices_by_ts.insert(100, slice1.clone());
        app.options_slices_by_ts.insert(200, slice2.clone());
        app.options_chain = Some(OptionsChain {
            underlying: "AAPL".into(),
            expirations: vec![exp1, exp2],
            selected_expiration_ts: 100,
            slice: slice1,
        });
        app.options_select_expiration(200);
        assert!(!app.options_inflight);
        assert_eq!(
            app.options_chain.as_ref().map(|c| c.selected_expiration_ts),
            Some(200)
        );
        assert_eq!(
            app.options_chain
                .as_ref()
                .and_then(|c| c.slice.calls.first().map(|x| x.strike)),
            Some(200.0)
        );
    }

    /// Issue #196 / §62 — [`App::on_theme_preset_committed`] must restyle baked Options tables.
    #[test]
    fn on_theme_preset_committed_updates_options_baked_stamp() {
        use crate::app::options::rebuild_options_display_cache;
        use crate::app::styles::ThemeStamp;
        use crate::config::theme::{Theme, ThemePreset};
        use crate::models::options::{
            Expiration, OptionContract, OptionRight, OptionsChain, OptionsChainSlice,
        };

        let mut app = App::new();
        app.symbol = "AAPL".into();
        app.config.theme = Some(Theme::from_preset(ThemePreset::Dark));
        app.options_chain = Some(OptionsChain {
            underlying: "AAPL".into(),
            expirations: vec![Expiration {
                ts: 1,
                label: "2026-06-20".into(),
            }],
            selected_expiration_ts: 1,
            slice: OptionsChainSlice {
                underlying: "AAPL".into(),
                expiration: Expiration {
                    ts: 1,
                    label: "2026-06-20".into(),
                },
                calls: vec![OptionContract {
                    symbol: "C220".into(),
                    strike: 220.0,
                    right: OptionRight::Call,
                    expiration_ts: 1,
                    bid: None,
                    ask: None,
                    last: None,
                    volume: None,
                    open_interest: None,
                    implied_volatility: None,
                    greeks: None,
                }],
                puts: vec![],
            },
        });
        rebuild_options_display_cache(&mut app);
        let dark_stamp = app.options_display.baked_theme_stamp;

        app.config.theme = Some(Theme::from_preset(ThemePreset::Light));
        app.on_theme_preset_committed();

        assert_ne!(app.options_display.baked_theme_stamp, dark_stamp);
        assert_eq!(
            app.options_display.baked_theme_stamp,
            Some(ThemeStamp::from_palette(&app.theme_palette_for_render()))
        );
    }

    fn historical_response_with_t_stride(
        bars: usize,
        t_stride_ms: u64,
    ) -> crate::models::historical::HistoricalResponse {
        use crate::models::historical::{HistoricalData, HistoricalResponse};
        const TS_BASE: u64 = 1_700_000_000_000;
        HistoricalResponse {
            ticker: "AAPL".into(),
            results: (0..bars)
                .map(|i| HistoricalData {
                    o: 100.0,
                    h: 101.0,
                    l: 99.0,
                    c: 100.5,
                    v: 1.0,
                    t: TS_BASE + i as u64 * t_stride_ms,
                    vw: 100.0,
                    n: None,
                })
                .collect(),
            status: "OK".into(),
            request_id: String::new(),
            count: bars as u32,
            ..Default::default()
        }
    }

    /// Issue #199 / §64.6 — stamp bumps on each successful historical apply (same-length refetch).
    #[test]
    fn historical_data_stamp_bumps_on_apply() {
        let mut app = App::new();
        app.symbol = "AAPL".to_string();
        app.time_range = TimeRange::Y1;
        let stamp0 = app.historical_data_stamp;
        app.apply_fetch_done(FetchDone::Historical {
            symbol: "AAPL".to_string(),
            time_range: TimeRange::Y1,
            result: Ok(historical_response_with_t_stride(10, 86_400_000)),
        });
        let stamp1 = app.historical_data_stamp;
        assert!(stamp1 > stamp0);
        app.apply_fetch_done(FetchDone::Historical {
            symbol: "AAPL".to_string(),
            time_range: TimeRange::Y1,
            result: Ok(historical_response_with_t_stride(10, 300_000)),
        });
        assert!(app.historical_data_stamp > stamp1);
    }
}
