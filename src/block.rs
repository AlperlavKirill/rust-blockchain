use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::transaction::Transaction;
use crate::utils::calculate_hash;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub hash: String,
    pub prev_hash: String,
    pub value: u64,
}

impl Block {
    pub fn new(
        index: u64,
        timestamp: u128,
        transactions: Vec<Transaction>,
        prev_hash: String,
        difficulty: usize,
    ) -> Self {
        let mut value = 0;
        let mut hash;

        loop {
            let data_to_hash = format!(
                "{}{}{:?}{}{}",
                index, timestamp, transactions, prev_hash, value
            );
            hash = calculate_hash(data_to_hash);

            if hash.starts_with(&"0".repeat(difficulty)) {
                break;
            }
            value += 1;
        }

        Block {
            index,
            timestamp,
            transactions,
            prev_hash,
            hash,
            value,
        }
    }

    pub fn calculate_hash(&self) -> String {
        calculate_hash(format!(
            "{}{}{:?}{}{}",
            self.index, self.timestamp, self.transactions, self.prev_hash, self.value
        ))
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block[{}]: {}. Previous block hash: {}\nTransactions:\n{}",
            self.index,
            self.hash,
            self.prev_hash,
            self.transactions
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
