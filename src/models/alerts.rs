use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

/// Applies threshold crossings using known last prices. Returns indices of alerts that **newly**
/// set `triggered` in this call (SPEC §18.9 — test hook).
pub fn process_alert_crossings(alerts: &mut [Alert], prices: &[(String, f64)]) -> Vec<usize> {
    let mut newly = Vec::new();
    for (i, alert) in alerts.iter_mut().enumerate() {
        if alert.triggered {
            continue;
        }
        let Some((_, price)) = prices
            .iter()
            .find(|(symbol, _)| symbol == &alert.symbol)
        else {
            continue;
        };
        let crossed = match alert.condition {
            AlertCondition::Above => *price > alert.price,
            AlertCondition::Below => *price < alert.price,
        };
        if crossed {
            alert.triggered = true;
            newly.push(i);
        }
    }
    newly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_alert_crossings_once_latches() {
        let mut alerts = vec![
            Alert::new("AAPL".into(), AlertCondition::Above, 1.0),
            Alert::new("MSFT".into(), AlertCondition::Below, 500.0),
        ];
        let prices = vec![("AAPL".into(), 150.0), ("MSFT".into(), 400.0)];
        let n1 = process_alert_crossings(&mut alerts, &prices);
        assert_eq!(n1, vec![0, 1]);
        assert!(alerts[0].triggered);
        assert!(alerts[1].triggered);

        let n2 = process_alert_crossings(&mut alerts, &prices);
        assert!(n2.is_empty());
    }
}
