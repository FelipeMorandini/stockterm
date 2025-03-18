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

pub enum Tab {
    StockView,
    Portfolio,
    Search,
    Settings,
    News,
    Charts,
}

pub struct App {
    pub config: Config,
    pub ticker_data: Option<TickerResponse>,
    pub historical_data: Option<HistoricalResponse>,
    pub search_results: Option<SymbolSearchResponse>,
    pub news_data: Option<NewsResponse>,
    pub should_quit: bool,
    pub symbol: String,
    pub portfolio: Vec<PortfolioItem>,
    pub portfolio_state: TableState,
    pub alerts: Vec<Alert>,
    pub alerts_state: TableState,
    pub active_tab: Tab,
    pub error_message: Option<String>,
    pub search_query: String,
    pub selected_index: usize,
}

impl App {
    pub fn new() -> App {
        let config = Config::load();
        App {
            config: config.clone(),
            ticker_data: None,
            historical_data: None,
            search_results: None,
            news_data: None,
            should_quit: false,
            symbol: String::from("AAPL"),
            portfolio: config.portfolio.clone(),
            portfolio_state: TableState::default(),
            alerts: Vec::new(),
            alerts_state: TableState::default(),
            active_tab: Tab::StockView,
            error_message: None,
            search_query: String::new(),
            selected_index: 0,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let events = Events::new();
        self.fetch_ticker_data().await; // Fetch initial data

        loop {
            draw(terminal, self)?;
            match events.next().map_err(|e| io::Error::new(io::ErrorKind::Other, e))? {
                Event::Input(input) => handle_event(self, input),
                Event::Tick => {
                    // Periodically update data based on active tab
                    match self.active_tab {
                        Tab::StockView => self.fetch_ticker_data().await,
                        Tab::Charts => self.fetch_historical_data().await,
                        Tab::News => self.fetch_news().await,
                        _ => {}
                    }
                }
            }



            if self.should_quit {
                return Ok(());
            }
        }
    }

    pub async fn fetch_ticker_data(&mut self) {
        if self.symbol.is_empty() {
            return;
        }

        match get_ticker_data(&self.symbol).await {
            Ok(data) => {
                self.ticker_data = Some(data);
                self.error_message = None;

                // Update portfolio prices if this symbol is in portfolio
                if let Some(item) = self.portfolio.iter_mut().find(|i| i.symbol == self.symbol) {
                    if let Some(ticker_data) = &self.ticker_data {
                        if !ticker_data.results.is_empty() {
                            item.current_price = Some(ticker_data.results[0].c);
                        }
                    }
                }
            },
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

        // Default to 1 month of data
        let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let from_date = (chrono::Local::now() - chrono::Duration::days(30))
            .format("%Y-%m-%d").to_string();

        match get_historical_data(&self.symbol, &from_date, &to_date, "day").await {
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

        match search_symbols(&self.search_query).await {
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

        match get_news(&self.symbol).await {
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
            Tab::Portfolio => Tab::Search,
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
            Tab::Search => Tab::Portfolio,
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
    }

    pub fn remove_from_portfolio(&mut self, index: usize) {
        if index < self.portfolio.len() {
            self.portfolio.remove(index);

            // Save to config
            self.config.portfolio = self.portfolio.clone();
            self.config.save();
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
