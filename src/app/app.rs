use crate::api::market_provider_for;
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
use ratatui::backend::Backend;
use ratatui::widgets::TableState;
use ratatui::Terminal;
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub enum Tab {
    StockView,
    Portfolio,
    Alerts,
    Search,
    News,
    Charts,
    Settings,
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
        result: Result<HistoricalResponse, String>,
    },
    News {
        symbol: String,
        result: Result<NewsResponse, String>,
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
    pub selected_index: usize,
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
    news_refresh_inflight: bool,
}

const MISSING_API_KEY_FOR_POLYGON_MSG: &str = "Polygon provider requires a non-empty `api_key` in ~/.stockterm.json or export STOCKTERM_API_KEY.";

const MAX_CONCURRENT_QUOTES: usize = 2;

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
            selected_index: 0,
            last_stock_network_poll: None,
            last_charts_network_poll: None,
            last_news_network_poll: None,
            stock_refresh_inflight: false,
            fetch_done_tx: None,
            stock_fetch_generation: 0,
            stock_refresh_pending: false,
            hist_refresh_inflight: false,
            news_refresh_inflight: false,
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
        tokio::spawn(async move {
            let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let from_date = (chrono::Local::now() - chrono::Duration::days(30))
                .format("%Y-%m-%d")
                .to_string();
            let provider = market_provider_for(cfg.provider);
            let result = provider
                .get_historical(&sym, &from_date, &to_date, "day", &cfg)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(FetchDone::Historical {
                symbol: sym,
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
            FetchDone::Historical { symbol, result } => {
                self.hist_refresh_inflight = false;
                self.last_charts_network_poll = Some(Instant::now());
                if symbol != self.symbol {
                    return;
                }
                match result {
                    Ok(data) => {
                        self.historical_data = Some(data);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Error fetching historical data: {err}"));
                        self.historical_data = None;
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
                        self.news_data = Some(data);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Error fetching news: {err}"));
                        self.news_data = None;
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

        let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let from_date = (chrono::Local::now() - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();

        let provider = market_provider_for(self.config.provider);
        match provider
            .get_historical(
                &self.symbol,
                &from_date,
                &to_date,
                "day",
                &self.config,
            )
            .await
        {
            Ok(data) => {
                self.historical_data = Some(data);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(format!("Error fetching historical data: {err}"));
                self.historical_data = None;
            }
        }
    }

    pub async fn search_symbols(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        if !self.provider_ready() {
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
            self.search_results = None;
            return;
        }

        let provider = market_provider_for(self.config.provider);
        match provider
            .search_symbols(&self.search_query, &self.config)
            .await
        {
            Ok(data) => {
                self.search_results = Some(data);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(format!("Error searching symbols: {err}"));
                self.search_results = None;
            }
        }
    }

    pub async fn fetch_news(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        if !self.provider_ready() {
            self.error_message = Some(MISSING_API_KEY_FOR_POLYGON_MSG.to_string());
            self.news_data = None;
            return;
        }

        let provider = market_provider_for(self.config.provider);
        match provider.get_news(&self.symbol, &self.config).await {
            Ok(data) => {
                self.news_data = Some(data);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(format!("Error fetching news: {err}"));
                self.news_data = None;
            }
        }
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
    use super::normalize_symbol;

    #[test]
    fn normalize_symbol_trims_and_uppercases() {
        assert_eq!(
            normalize_symbol("  aapl  ").as_deref(),
            Some("AAPL")
        );
        assert_eq!(normalize_symbol("   "), None);
        assert_eq!(normalize_symbol(""), None);
    }
}
