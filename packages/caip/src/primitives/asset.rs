use serde::{Deserialize, Serialize};

use crate::{AssetId, Money};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub money: Money,
}

impl Asset {
    pub fn new(id: AssetId, money: Money) -> Self {
        Self { id, money }
    }

    /// Calculates the USD value of this asset given the price per token.
    pub fn usd_value(&self, price_per_token: f64) -> f64 {
        self.money.to_f64() * price_per_token
    }
}
