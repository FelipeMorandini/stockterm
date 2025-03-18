use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SymbolSearchResponse {
    pub status: String,
    pub count: u32,
    pub results: Vec<SymbolResult>,
}

#[derive(Deserialize, Debug)]
pub struct SymbolResult {
    pub ticker: String,
    pub name: String,
    pub market: String,
    pub locale: String,
    pub primary_exchange: String,
    pub type_: String,
    pub active: bool,
    pub currency_name: String,
    pub cik: Option<String>,
    pub composite_figi: Option<String>,
    pub share_class_figi: Option<String>,
    pub last_updated_utc: String,
}