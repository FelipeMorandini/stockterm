use crate::models::ticker::TickerResponse;
use crate::models::historical::HistoricalResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::news::NewsResponse;
use crate::config::Config;
use reqwest;

const API_KEY: &str = "YOUR_POLYGON_API_KEY";
const BASE_URL: &str = "https://api.polygon.io";

pub async fn get_ticker_data(symbol: &str, config: &Config) -> Result<TickerResponse, reqwest::Error> {
    let url = format!(
        "{}/v2/aggs/ticker/{}/range/1/day/2023-01-01/2023-12-31?apiKey={}",
        BASE_URL, symbol, &config.api_key
    );

    let response = reqwest::get(&url).await?;
    let ticker_data: TickerResponse = response.json().await?;
    Ok(ticker_data)
}

pub async fn get_historical_data(
    symbol: &str,
    from_date: &str,
    to_date: &str,
    timespan: &str
) -> Result<HistoricalResponse, reqwest::Error> {
    let url = format!(
        "{}/v2/aggs/ticker/{}/range/1/{}/{}/{}?apiKey={}",
        BASE_URL, symbol, timespan, from_date, to_date, API_KEY
    );

    let response = reqwest::get(&url).await?;
    let historical_data: HistoricalResponse = response.json().await?;
    Ok(historical_data)
}

pub async fn search_symbols(query: &str) -> Result<SymbolSearchResponse, reqwest::Error> {
    let url = format!(
        "{}/v3/reference/tickers?search={}&active=true&apiKey={}",
        BASE_URL, query, API_KEY
    );

    let response = reqwest::get(&url).await?;
    let search_results: SymbolSearchResponse = response.json().await?;
    Ok(search_results)
}

pub async fn get_news(symbol: &str) -> Result<NewsResponse, reqwest::Error> {
    let url = format!(
        "{}/v2/reference/news?ticker={}&apiKey={}",
        BASE_URL, symbol, API_KEY
    );

    let response = reqwest::get(&url).await?;
    let news: NewsResponse = response.json().await?;
    Ok(news)
}
