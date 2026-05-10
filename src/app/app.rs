use crate::api::market_provider_for;
use crate::api::HistoricalQuery;
use crate::app::charts::{viewport_zoom_in, viewport_zoom_out, ChartDisplayMode, ChartViewport};
use crate::app::event::{spawn_event_thread, Event};
use crate::app::handlers::handle_event;
use crate::app::ui::draw;
use crate::config::{Config, MarketProviderKind};
use crate::models::alerts::Alert;
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::portfolio::PortfolioItem;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;
use crate::models::time_range::TimeRange;
use ratatui::backend::Backend;
use ratatui::widgets::{ListState, TableState};
use ratatui::Terminal;
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    StockView,
    Portfolio,
    Alerts,
    Search,
    News,
    Charts,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsEdit {
    RefreshRate,
    DefaultSymbol,
}

/// Outcomes from background HTTP tasks (never awaited on the draw/input hot path).
pub enum FetchDone {
    Stock {
        generation: u64,
        quotes: HashMap<String, TickerResponse>,
        errors: Vec<String>,
    },
    Historical {
        symbol: String,
        time_range: TimeRange,
        result: Result<HistoricalResponse, String>,
    },
    News {
        symbol: String,
        result: Result<NewsResponse, String>,
    },
    Search {
        generation: u64,
        query: String,
        result: Result<SymbolSearchResponse, String>,
    },
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
    pub error_message: Option<String>,
    pub search_query: String,
    pub search_table_state: TableState,
    pub search_request_generation: u64,
    pub search_refresh_inflight: bool,
    search_debounce_deadline: Option<Instant>,
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
    stock_fetch_generation: u64,
    stock_refresh_pending: bool,
    hist_refresh_inflight: bool,
    pub news_refresh_inflight: bool,
    /// Charts tab: selected window (Issue #9).
    pub time_range: TimeRange,
    /// Charts tab: pan/zoom indices into `historical_data.results` (Issue #8).
    pub chart_viewport: ChartViewport,
    /// Line vs candlestick rendering (Issue #7).
    pub chart_mode: ChartDisplayMode,
}

const MISSING_API_KEY_FOR_POLYGON_MSG: &str = "Polygon provider requires a non-empty `api_key` in ~/.stockterm.json or export STOCKTERM_API_KEY.";

const MAX_CONCURRENT_QUOTES: usize = 2;

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(250);

/// Rows in the Settings tab (refresh, default symbol, theme, provider, keymap).
pub const SETTINGS_ROW_COUNT: usize = 5;

const SETTINGS_SAVED_FLASH: Duration = Duration::from_secs(2);

/// Trim and uppercase ticker input; returns `None` if empty after trim.
pub fn normalize_symbol(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    Some(t.to_uppercase())
}

async fn run_stock_quote_batch(
    generation: u64,
    symbols: Vec<String>,
    config: Config,
) -> FetchDone {
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
                    errors.push(format!("{sym}: {msg}"));
                    continue;
                }
                if data.ticker.is_empty() {
                    data.ticker = sym.clone();
                }
                quotes.insert(sym, data);
            }
            Ok((sym, Err(e))) => errors.push(format!("{sym}: {e}")),
            Err(e) => errors.push(format!("task join: {e}")),
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
        let config = Config::load();
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
            normalize_symbol(&config.default_symbol).unwrap_or_else(|| "AAPL".to_string())
        };

        let mut watchlist_state = TableState::default();
        if !watchlist.is_empty() {
            watchlist_state.select(Some(0));
        }

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
            active_tab: Tab::StockView,
            error_message: None,
            search_query: String::new(),
            search_table_state: TableState::default(),
            search_request_generation: 0,
            search_refresh_inflight: false,
            search_debounce_deadline: None,
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
            stock_fetch_generation: 0,
            stock_refresh_pending: false,
            hist_refresh_inflight: false,
            news_refresh_inflight: false,
            time_range: TimeRange::default(),
            chart_viewport: ChartViewport::default(),
            chart_mode: ChartDisplayMode::default(),
        };

        if !app.portfolio.is_empty() {
            app.portfolio_state.select(Some(0));
        }
        if !app.alerts.is_empty() {
            app.alerts_state.select(Some(0));
        }

        app
    }

    fn provider_ready(&self) -> bool {
        match self.config.provider {
            MarketProviderKind::Yahoo => true,
            MarketProviderKind::Polygon => !self.config.effective_api_key().is_empty(),
        }
    }

    fn data_poll_interval(&self) -> Duration {
        let secs = match self.config.refresh_rate {
            0 => 30,
            s => s,
        };
        Duration::from_secs(secs.max(5))
    }

    fn collect_symbols_for_quote_fetch(&self) -> Vec<String> {
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
        out
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
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
            self.ticker_data = None;
            return;
        }

        self.stock_refresh_inflight = true;
        self.stock_fetch_generation += 1;
        let generation = self.stock_fetch_generation;
        let cfg = self.config.clone();

        tokio::spawn(async move {
            let done = run_stock_quote_batch(generation, symbols, cfg).await;
            let _ = tx.send(done);
        });
    }

    fn apply_stock_fetch_done(&mut self, generation: u64, quotes: HashMap<String, TickerResponse>, errors: Vec<String>) {
        if generation != self.stock_fetch_generation {
            return;
        }

        self.stock_refresh_inflight = false;
        self.last_stock_network_poll = Some(Instant::now());

        for (k, v) in quotes {
            self.watchlist_quotes.insert(k, v);
        }

        self.ticker_data = self.watchlist_quotes.get(&self.symbol).cloned();

        if self.ticker_data.is_none() && !errors.is_empty() {
            self.error_message = Some(errors.join("; "));
        } else if !errors.is_empty() {
            self.error_message = Some(format!("Some quotes failed: {}", errors.join("; ")));
        } else {
            self.error_message = None;
        }

        for item in &mut self.portfolio {
            if let Some(resp) = self.watchlist_quotes.get(&item.symbol) {
                if let Some(bar) = resp.latest_result() {
                    item.current_price = Some(bar.c);
                }
            }
        }

        self.check_alerts();

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
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
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
            let result = provider
                .get_historical(&sym, &hq, &cfg)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(FetchDone::Historical {
                symbol: sym,
                time_range: tr,
                result,
            });
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
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
            self.news_data = None;
            return;
        }

        let Some(tx) = self.fetch_done_tx.clone() else {
            return;
        };

        self.news_refresh_inflight = true;
        let sym = self.symbol.clone();
        let cfg = self.config.clone();
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.get_news(&sym, &cfg).await.map_err(|e| e.to_string());
            let _ = tx.send(FetchDone::News {
                symbol: sym,
                result,
            });
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
    pub fn search_esc_reset(&mut self) {
        self.search_query.clear();
        self.search_results = None;
        self.search_table_state.select(None);
        self.search_debounce_deadline = None;
        self.search_request_generation = self.search_request_generation.wrapping_add(1);
        self.error_message = None;
    }

    fn try_spawn_search_tick(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results = None;
            self.search_debounce_deadline = None;
            return;
        }
        if !self.provider_ready() {
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
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
        tokio::spawn(async move {
            let provider = market_provider_for(cfg.provider);
            let result = provider.search_symbols(&query, &cfg).await.map_err(|e| e.to_string());
            let _ = tx.send(FetchDone::Search {
                generation,
                query,
                result,
            });
        });
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

    pub fn news_try_open_selected(&mut self) {
        use crate::app::open_url::open_article_url;
        let Some(data) = self.news_data.as_ref() else {
            return;
        };
        let n = data.results.len();
        if n == 0 {
            return;
        }
        let i = self.news_list_state.selected().unwrap_or(0).min(n - 1);
        let Some(item) = data.results.get(i) else {
            return;
        };
        if let Err(e) = open_article_url(&item.article_url) {
            self.error_message = Some(format!("Could not open URL: {e}"));
        } else {
            self.error_message = None;
        }
    }

    pub fn settings_row_prev(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        self.settings_row = self.settings_row.saturating_sub(1);
    }

    pub fn settings_row_next(&mut self) {
        if self.settings_editing.is_some() {
            return;
        }
        self.settings_row = (self.settings_row + 1).min(SETTINGS_ROW_COUNT - 1);
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
                if let Err(e) = self.config.try_save() {
                    self.error_message = Some(format!("Failed to save settings: {e}"));
                } else {
                    self.error_message = None;
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
                if let Err(e) = self.config.try_save() {
                    self.error_message = Some(format!("Failed to save settings: {e}"));
                } else {
                    self.error_message = None;
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
            _ => {}
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
                        );
                        self.historical_data = Some(data);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Error fetching historical data: {err}"));
                        self.historical_data = None;
                        self.chart_viewport = ChartViewport::default();
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
                        self.error_message = None;
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
                        self.error_message = Some(format!("Error fetching news: {err}"));
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
                        self.error_message = None;
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
                        self.error_message = Some(format!("Error searching symbols: {e}"));
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

        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        spawn_event_thread(event_tx);

        self.request_immediate_stock_poll();

        loop {
            draw(terminal, self)?;

            if self.should_quit {
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
                            }
                        }
                        Some(Event::Tick) => self.on_background_tick(),
                        None => return Ok(()),
                    }
                }
                done = fetch_rx.recv() => {
                    if let Some(msg) = done {
                        self.apply_fetch_done(msg);
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
    pub fn sync_watchlist_selection_to_symbol(&mut self) {
        if let Some(pos) = self.watchlist.iter().position(|s| s == &self.symbol) {
            self.watchlist_state.select(Some(pos));
        }
    }

    pub fn add_current_to_watchlist(&mut self) {
        let Some(sym) = normalize_symbol(&self.symbol) else {
            return;
        };
        if self.watchlist.iter().any(|s| s == &sym) {
            return;
        }
        self.watchlist.push(sym.clone());
        self.symbol = sym;
        self.config.watchlist = self.watchlist.clone();
        if let Err(e) = self.config.try_save() {
            self.error_message = Some(format!("Failed to save watchlist: {e}"));
        } else {
            self.error_message = None;
        }
        self.watchlist_state.select(Some(self.watchlist.len().saturating_sub(1)));
        self.notify_symbol_changed_for_news();
    }

    pub fn remove_selected_watchlist_row(&mut self) {
        let Some(selected) = self.watchlist_state.selected() else {
            return;
        };
        if selected >= self.watchlist.len() {
            return;
        }
        self.watchlist.remove(selected);
        self.watchlist_quotes.retain(|k, _| self.watchlist.contains(k));
        self.config.watchlist = self.watchlist.clone();
        if let Err(e) = self.config.try_save() {
            self.error_message = Some(format!("Failed to save watchlist: {e}"));
        } else {
            self.error_message = None;
        }

        if self.watchlist.is_empty() {
            self.watchlist_state.select(None);
        } else {
            let ni = selected.min(self.watchlist.len() - 1);
            self.watchlist_state.select(Some(ni));
            self.symbol = self.watchlist[ni].clone();
        }

        self.ticker_data = self
            .watchlist_quotes
            .get(&self.symbol)
            .cloned();
        self.notify_symbol_changed_for_news();
    }

    pub fn watchlist_select_prev(&mut self) {
        if self.watchlist.is_empty() {
            return;
        }
        match self.watchlist_state.selected() {
            None => self
                .watchlist_state
                .select(Some(self.watchlist.len().saturating_sub(1))),
            Some(i) if i > 0 => self.watchlist_state.select(Some(i - 1)),
            _ => {}
        }
        if let Some(i) = self.watchlist_state.selected() {
            self.symbol = self.watchlist[i].clone();
        }
        self.notify_symbol_changed_for_news();
    }

    pub fn watchlist_select_next(&mut self) {
        if self.watchlist.is_empty() {
            return;
        }
        match self.watchlist_state.selected() {
            None => self.watchlist_state.select(Some(0)),
            Some(i) if i < self.watchlist.len().saturating_sub(1) => {
                self.watchlist_state.select(Some(i + 1));
            }
            _ => {}
        }
        if let Some(i) = self.watchlist_state.selected() {
            self.symbol = self.watchlist[i].clone();
        }
        self.notify_symbol_changed_for_news();
    }

    pub async fn fetch_historical_data(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        if !self.provider_ready() {
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
            self.historical_data = None;
            return;
        }

        let params = self
            .time_range
            .historical_params(chrono::Local::now());
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

        let provider = market_provider_for(self.config.provider);
        match provider.get_historical(&self.symbol, &hq, &self.config).await
        {
            Ok(data) => {
                let prev = self.historical_data.as_ref();
                self.chart_viewport = crate::app::charts::chart_viewport_after_refresh(
                    prev,
                    self.chart_viewport,
                    &data,
                );
                self.historical_data = Some(data);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(format!("Error fetching historical data: {err}"));
                self.historical_data = None;
                self.chart_viewport = ChartViewport::default();
            }
        }
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
            self.error_message = None;
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
        self.active_tab = match self.active_tab {
            Tab::StockView => Tab::Portfolio,
            Tab::Portfolio => Tab::Alerts,
            Tab::Alerts => Tab::Search,
            Tab::Search => Tab::News,
            Tab::News => Tab::Charts,
            Tab::Charts => Tab::Settings,
            Tab::Settings => Tab::StockView,
        };
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::StockView => Tab::Settings,
            Tab::Portfolio => Tab::StockView,
            Tab::Alerts => Tab::Portfolio,
            Tab::Search => Tab::Alerts,
            Tab::News => Tab::Search,
            Tab::Charts => Tab::News,
            Tab::Settings => Tab::Charts,
        };
    }

    pub fn add_to_portfolio(&mut self, shares: f64, purchase_price: f64) {
        if self.symbol.is_empty() {
            return;
        }

        if let Some(item) = self.portfolio.iter_mut().find(|i| i.symbol == self.symbol) {
            item.shares += shares;
            let total_shares = item.shares;
            let existing_cost = (total_shares - shares) * item.purchase_price;
            let new_cost = shares * purchase_price;
            item.purchase_price = (existing_cost + new_cost) / total_shares;
        } else {
            self.portfolio.push(PortfolioItem::new(
                self.symbol.clone(),
                shares,
                purchase_price,
            ));
        }

        self.config.portfolio = self.portfolio.clone();
        self.config.save();

        if !self.portfolio.is_empty() && self.portfolio_state.selected().is_none() {
            self.portfolio_state.select(Some(self.portfolio.len() - 1));
        }
    }

    pub fn remove_from_portfolio(&mut self, index: usize) {
        if index >= self.portfolio.len() {
            return;
        }

        self.portfolio.remove(index);
        self.config.portfolio = self.portfolio.clone();
        self.config.save();

        if self.portfolio.is_empty() {
            self.portfolio_state.select(None);
        } else if let Some(mut sel) = self.portfolio_state.selected() {
            if index < sel {
                sel -= 1;
            } else if index == sel {
                sel = sel.min(self.portfolio.len() - 1);
            }
            self.portfolio_state.select(Some(sel));
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
    use super::{normalize_symbol, search_result_matches_current};

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
