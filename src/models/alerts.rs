use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AlertCondition {
    Above,
    Below,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alert {
    pub symbol: String,
    pub condition: AlertCondition,
    pub price: f64,
    pub triggered: bool,
}

impl Alert {
    pub fn new(symbol: String, condition: AlertCondition, price: f64) -> Self {
        Self {
            symbol,
            condition,
            price,
            triggered: false,
        }
    }

    pub fn is_triggered(&self, current_price: f64) -> bool {
        match self.condition {
            AlertCondition::Above => current_price > self.price,
            AlertCondition::Below => current_price < self.price,
        }
    }
}
