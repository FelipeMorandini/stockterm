use chrono::{Duration, Local};
use crate::config::Config;
use crate::models::historical::HistoricalResponse;
use crate::models::news::NewsResponse;
use crate::models::search::SymbolSearchResponse;
use crate::models::ticker::TickerResponse;
use reqwest;
use urlencoding::encode;

const BASE_URL: &str = "https://api.polygon.io";

fn enc(s: &str) -> String {
    encode(s).into_owned()
}

pub async fn get_ticker_data(
    symbol: &str,
    config: &Config,
) -> Result<TickerResponse, reqwest::Error> {
    let key = config.effective_api_key();
    let to = Local::now().format("%Y-%m-%d").to_string();
    let from = (Local::now() - Duration::days(30)).format("%Y-%m-%d").to_string();
    let url = format!(
        "{}/v2/aggs/ticker/{}/range/1/day/{}/{}?adjusted=true&sort=desc&limit=120&apiKey={}",
        BASE_URL,
        enc(symbol),
        enc(&from),
        enc(&to),
        enc(key.as_ref())
    );

    let response = reqwest::get(&url).await?.error_for_status()?;
    let ticker_data: TickerResponse = response.json().await?;
    Ok(ticker_data)
}

pub async fn get_historical_data(
    symbol: &str,
    from_date: &str,
    to_date: &str,
    timespan: &str,
    config: &Config,
) -> Result<HistoricalResponse, reqwest::Error> {
    let key = config.effective_api_key();
    let url = format!(
        "{}/v2/aggs/ticker/{}/range/1/{}/{}/{}?apiKey={}",
        BASE_URL,
        enc(symbol),
        enc(timespan),
        enc(from_date),
        enc(to_date),
        enc(key.as_ref())
    );

    let response = reqwest::get(&url).await?.error_for_status()?;
    let historical_data: HistoricalResponse = response.json().await?;
    Ok(historical_data)
}

pub async fn search_symbols(
    query: &str,
    config: &Config,
) -> Result<SymbolSearchResponse, reqwest::Error> {
    let key = config.effective_api_key();
    let url = format!(
        "{}/v3/reference/tickers?search={}&active=true&apiKey={}",
        BASE_URL,
        enc(query),
        enc(key.as_ref())
    );

    let response = reqwest::get(&url).await?.error_for_status()?;
    let search_results: SymbolSearchResponse = response.json().await?;
    Ok(search_results)
}

pub async fn get_news(symbol: &str, config: &Config) -> Result<NewsResponse, reqwest::Error> {
    let key = config.effective_api_key();
    let url = format!(
        "{}/v2/reference/news?ticker={}&apiKey={}",
        BASE_URL,
        enc(symbol),
        enc(key.as_ref())
    );

    let response = reqwest::get(&url).await?.error_for_status()?;
    let news: NewsResponse = response.json().await?;
    Ok(news)
}
