use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::blockchain::TRANSACTION_FEE;

// todo add verification and validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}: {} (комиссия {})", self.from, self.to, self.amount, TRANSACTION_FEE)
    }
}