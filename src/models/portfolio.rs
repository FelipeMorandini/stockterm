use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortfolioItem {
    pub symbol: String,
    pub shares: f64,
    pub purchase_price: f64,
    pub current_price: Option<f64>,
    pub purchase_date: Option<String>,
    pub notes: Option<String>,
}

impl PortfolioItem {
    pub fn new(symbol: String, shares: f64, purchase_price: f64) -> Self {
        Self {
            symbol,
            shares,
            purchase_price,
            current_price: None,
            purchase_date: None,
            notes: None,
        }
    }

    pub fn market_value(&self) -> Option<f64> {
        self.current_price.map(|price| price * self.shares)
    }

    pub fn cost_basis(&self) -> f64 {
        self.purchase_price * self.shares
    }

    pub fn profit_loss(&self) -> Option<f64> {
        self.market_value().map(|value| value - self.cost_basis())
    }

    pub fn profit_loss_percent(&self) -> Option<f64> {
        self.profit_loss().map(|pl| (pl / self.cost_basis()) * 100.0)
    }
}
