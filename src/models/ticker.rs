use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TickerResponse {
    pub ticker: String,
    pub results: Vec<TickerResult>,
}

#[derive(Deserialize, Debug)]
pub struct TickerResult {
    pub o: f64, // Open
    pub h: f64, // High
    pub l: f64, // Low
    pub c: f64, // Close
    pub v: u64, // Volume
    pub t: u64, // Timestamp
}