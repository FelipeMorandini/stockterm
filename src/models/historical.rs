use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HistoricalResponse {
    #[serde(default)]
    pub ticker: String,
    #[serde(default)]
    pub results: Vec<HistoricalData>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub count: u32,
}

#[derive(Deserialize, Debug)]
pub struct HistoricalData {
    pub o: f64, // Open
    pub h: f64, // High
    pub l: f64, // Low
    pub c: f64, // Close
    pub v: f64, // Volume (Polygon may return fractional values)
    pub t: u64, // Timestamp
    pub vw: f64, // Volume weighted average price
    pub n: Option<u64>, // Number of transactions
}