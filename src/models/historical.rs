use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HistoricalResponse {
    pub ticker: String,
    pub results: Vec<HistoricalData>,
    pub status: String,
    pub request_id: String,
    pub count: u32,
}

#[derive(Deserialize, Debug)]
pub struct HistoricalData {
    pub o: f64, // Open
    pub h: f64, // High
    pub l: f64, // Low
    pub c: f64, // Close
    pub v: u64, // Volume
    pub t: u64, // Timestamp
    pub vw: f64, // Volume weighted average price
    pub n: Option<u64>, // Number of transactions
}