use crate::config::Config;
use crate::models::ticker::TickerResponse;
use crate::models::historical::HistoricalResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::news::NewsResponse;
use crate::models::portfolio::PortfolioItem;
use crate::models::alerts::Alert;
use crate::api::polygon::{get_ticker_data, get_historical_data, search_symbols, get_news};
use crate::app::ui::draw;
use crate::app::event::{Event, Events};
use crate::app::handlers::handle_event;
use ratatui::{
    backend::Backend,
    Terminal,
    widgets::TableState,
};
use std::io;
use std::time::{Duration, Instant};

pub enum Tab {
    StockView,
    Portfolio,
    Alerts,
    Search,
    News,
    Charts,
    Settings,
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
    pub portfolio: Vec<PortfolioItem>,
    pub portfolio_state: TableState,
    pub alerts: Vec<Alert>,
    pub alerts_state: TableState,
    pub active_tab: Tab,
    pub error_message: Option<String>,
    pub search_query: String,
    pub selected_index: usize,
    /// Throttle tick-driven network calls so the UI thread stays responsive.
    last_stock_network_poll: Option<Instant>,
    last_charts_network_poll: Option<Instant>,
    last_news_network_poll: Option<Instant>,
}

const MISSING_POLYGON_KEY_MSG: &str = "Missing Polygon API key. Set non-empty `api_key` in ~/.stockterm.json or export STOCKTERM_API_KEY. (401 with an empty apiKey= query means no key was configured.)";

impl App {
    pub fn new() -> App {
        let config = Config::load();
        let portfolio = config.portfolio.clone();
        let alerts = config.alerts.clone();

        let mut app = App {
            config: config.clone(),
            ticker_data: None,
            historical_data: None,
            search_results: None,
            news_data: None,
            should_quit: false,
            should_fetch_ticker: false,
            symbol: String::from("AAPL"),
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
        };

        if !app.portfolio.is_empty() {
            app.portfolio_state.select(Some(0));
        }
        if !app.alerts.is_empty() {
            app.alerts_state.select(Some(0));
        }

        app
    }

    fn polygon_key_configured(&self) -> bool {
        !self.config.effective_api_key().is_empty()
    }

    fn data_poll_interval(&self) -> Duration {
        let secs = match self.config.refresh_rate {
            0 => 30,
            s => s,
        };
        Duration::from_secs(secs.max(5))
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let events = Events::new();
        self.fetch_ticker_data().await;
        self.last_stock_network_poll = Some(Instant::now());

        loop {
            draw(terminal, self)?;

            match events.next().map_err(io::Error::other)? {
                Event::Input(input) => handle_event(self, input).await,
                Event::Tick => {
                    let interval = self.data_poll_interval();
                    let now = Instant::now();
                    match self.active_tab {
                        Tab::StockView | Tab::Alerts
                            if self.last_stock_network_poll.is_none_or(|t| {
                                now.duration_since(t) >= interval
                            }) =>
                        {
                            self.fetch_ticker_data().await;
                            self.last_stock_network_poll = Some(Instant::now());
                        }
                        Tab::Charts
                            if self.last_charts_network_poll.is_none_or(|t| {
                                now.duration_since(t) >= interval
                            }) =>
                        {
                            self.fetch_historical_data().await;
                            self.last_charts_network_poll = Some(Instant::now());
                        }
                        Tab::News
                            if self.last_news_network_poll.is_none_or(|t| {
                                now.duration_since(t) >= interval
                            }) =>
                        {
                            self.fetch_news().await;
                            self.last_news_network_poll = Some(Instant::now());
                        }
                        _ => {}
                    }
                }
            }

            if self.should_fetch_ticker {
                self.fetch_ticker_data().await;
                self.should_fetch_ticker = false;
                self.last_stock_network_poll = Some(Instant::now());
            }

            if self.should_quit {
                return Ok(());
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
    pub async fn fetch_ticker_data(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        if !self.polygon_key_configured() {
            self.error_message = Some(MISSING_POLYGON_KEY_MSG.to_string());
            self.ticker_data = None;
            return;
        }

        match get_ticker_data(&self.symbol, &self.config).await {
            Ok(mut data) => {
                if let Some(msg) = data.api_error_message() {
                    self.error_message = Some(msg);
                    self.ticker_data = None;
                    return;
                }
                if data.ticker.is_empty() {
                    data.ticker = self.symbol.clone();
                }
                self.ticker_data = Some(data);
                self.error_message = None;

                if let Some(item) = self.portfolio.iter_mut().find(|i| i.symbol == self.symbol) {
                    if let Some(ticker_data) = &self.ticker_data {
                        if let Some(bar) = ticker_data.latest_result() {
                            item.current_price = Some(bar.c);
                        }
                    }
                }
                self.check_alerts();
            }
            Err(err) => {
                self.error_message = Some(format!("Error fetching ticker data: {}", err));
                self.ticker_data = None;
            }
        }
    }

    pub async fn fetch_historical_data(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        if !self.polygon_key_configured() {
            self.error_message = Some(MISSING_POLYGON_KEY_MSG.to_string());
            self.historical_data = None;
            return;
        }

        // Default to 1 month of data
        let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let from_date = (chrono::Local::now() - chrono::Duration::days(30))
            .format("%Y-%m-%d").to_string();

        match get_historical_data(&self.symbol, &from_date, &to_date, "day", &self.config).await {
            Ok(data) => {
                self.historical_data = Some(data);
                self.error_message = None;
            },
            Err(err) => {
                self.error_message = Some(format!("Error fetching historical data: {}", err));
                self.historical_data = None;
            }
        }
    }

    pub async fn search_symbols(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        if !self.polygon_key_configured() {
            self.error_message = Some(MISSING_POLYGON_KEY_MSG.to_string());
            self.search_results = None;
            return;
        }

        match search_symbols(&self.search_query, &self.config).await {
            Ok(data) => {
                self.search_results = Some(data);
                self.error_message = None;
            },
            Err(err) => {
                self.error_message = Some(format!("Error searching symbols: {}", err));
                self.search_results = None;
            }
        }
    }

    pub async fn fetch_news(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        if !self.polygon_key_configured() {
            self.error_message = Some(MISSING_POLYGON_KEY_MSG.to_string());
            self.news_data = None;
            return;
        }

        match get_news(&self.symbol, &self.config).await {
            Ok(data) => {
                self.news_data = Some(data);
                self.error_message = None;
            },
            Err(err) => {
                self.error_message = Some(format!("Error fetching news: {}", err));
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

        // Check if already in portfolio
        if let Some(item) = self.portfolio.iter_mut().find(|i| i.symbol == self.symbol) {
            item.shares += shares;
            // Calculate weighted average purchase price
            let total_shares = item.shares;
            let existing_cost = (total_shares - shares) * item.purchase_price;
            let new_cost = shares * purchase_price;
            item.purchase_price = (existing_cost + new_cost) / total_shares;
        } else {
            self.portfolio.push(PortfolioItem::new(
                self.symbol.clone(),
                shares,
                purchase_price
            ));
        }

        // Save to config
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
        self.portfolio.iter()
            .filter_map(|item| item.market_value())
            .sum()
    }

    pub fn calculate_portfolio_cost(&self) -> f64 {
        self.portfolio.iter()
            .map(|item| item.cost_basis())
            .sum()
    }

    pub fn calculate_portfolio_profit_loss(&self) -> f64 {
        self.calculate_portfolio_value() - self.calculate_portfolio_cost()
    }
}
