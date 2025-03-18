use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NewsResponse {
    pub status: String,
    pub count: u32,
    pub results: Vec<NewsItem>,
}

#[derive(Deserialize, Debug)]
pub struct NewsItem {
    pub id: String,
    pub publisher: Publisher,
    pub title: String,
    pub author: Option<String>,
    pub published_utc: String,
    pub article_url: String,
    pub tickers: Vec<String>,
    pub amp_url: Option<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Publisher {
    pub name: String,
    pub homepage_url: String,
    pub logo_url: String,
    pub favicon_url: String,
}
