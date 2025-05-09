use crate::block::Block;
use crate::transaction::Transaction;
use crate::utils::now;
use crate::wallet::Wallet;

pub struct Blockchain {
    db: sled::Db,
    difficulty: usize,
}

const BALANCE_PREFIX: &str = "balance:";
const BLOCK_PREFIX: &str = "block:";
const NETWORK_ADDRESS: &str = "network";

const BLOCK_REWARD: f64 = 5.0;
pub const TRANSACTION_FEE: f64 = 0.01;

// todo
//  - транзакция попадает в пул, откуда ее может взять в блок майнер с кастомной комиссией


impl Blockchain {
    pub fn new(path: &str, difficulty: usize) -> Self {
        let db = sled::open(path).expect("Failed to open database");
        let blockchain = Blockchain { db, difficulty };

        if blockchain.latest_block().is_none() {
            println!("Инициализация первого блока...");
            let initial_tx = Transaction {
                from: NETWORK_ADDRESS.to_string(),
                to: "02c9cfea78bd540fae61e64ba3b848f691aeabbd2a36b02a7dd57513752441523b".to_string(),
                amount: 100.0,
                signature: vec![],
                public_key: vec![],
            };
            let initial_block = Block::new(
                0,
                now(),
                vec![initial_tx.clone()],
                "0".to_owned(),
                difficulty,
            );
            let serialized_block = bincode::serialize(&initial_block).unwrap();
            let key = format!("{}{}", BLOCK_PREFIX, initial_block.index);
            blockchain.db.insert(key, serialized_block).unwrap();
            blockchain.db.flush().unwrap();

            blockchain.save_balance(&initial_tx.to, initial_tx.amount);
        }

        blockchain
    }

    pub fn load_blockchain(&self) -> Vec<Block> {
        let mut blocks = vec![];
        for block_result in self.db.scan_prefix(BLOCK_PREFIX) {
            let (_, val) = block_result.unwrap();
            let block: Block = bincode::deserialize(&val).unwrap();
            blocks.push(block);
        }
        blocks.sort_by_key(|b| b.index);
        blocks
    }

    pub fn latest_block(&self) -> Option<Block> {
        self.load_blockchain().last().cloned()
    }

    pub fn add_block(&mut self, miner_address: &str, transactions: Vec<Transaction>) {
        for tx in &transactions {
            if tx.from != NETWORK_ADDRESS {

                let transaction_data = format!("{}{}{}", tx.from, tx.to, tx.amount);
                if !Wallet::verify(&tx.public_key, transaction_data.as_bytes(), &tx.signature) {
                    println!("Ошибка. Неверная подпись в транзакции {} -> {}: {}", tx.from, tx.to, tx.amount);
                    return;
                }

                let balance = self.load_balance(&tx.from);
                let total_amount = tx.amount + TRANSACTION_FEE;
                if total_amount > balance {
                    println!(
                        "Ошибка. Недостаточно средств для транзакции {} -> {}: {} с комиссией {}",
                        tx.from, tx.to, tx.amount, TRANSACTION_FEE
                    );
                    return;
                }
            }
        }

        let total_fees = transactions
            .iter()
            .map(|_| TRANSACTION_FEE)
            .sum::<f64>();

        let mut block_transactions = vec![];

        block_transactions.push(Transaction {
            from: NETWORK_ADDRESS.to_string(),
            to: miner_address.to_string(),
            amount: BLOCK_REWARD + total_fees,
            signature: vec![],
            public_key: vec![],
        });

        block_transactions.extend(transactions.clone());

        let last_block = self.latest_block().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            now(),
            block_transactions.clone(),
            last_block.hash.clone(),
            self.difficulty,
        );

        let serialized_block = bincode::serialize(&new_block).unwrap();
        let block_key = format!("{}{}", BLOCK_PREFIX, new_block.index);
        self.db.insert(block_key, serialized_block).unwrap();
        self.db.flush().unwrap();

        for tx in transactions {
            if tx.from != NETWORK_ADDRESS {
                let balance = self.load_balance(&tx.from);
                self.save_balance(&tx.from, balance - tx.amount);
            }
            self.save_balance(&tx.to, self.load_balance(&tx.to) + tx.amount);
        }
    }

    pub fn add_block_from_p2p(&mut self, block: Block) -> bool {
        let latest_block = self.latest_block().unwrap();

        if block.index != latest_block.index + 1 {
            return false;
        }

        if block.prev_hash != latest_block.hash {
            return false;
        }

        if block.hash != block.calculate_hash() {
            return false;
        }

        if !block.hash.starts_with(&"0".repeat(self.difficulty)) {
            return false;
        }

        let encoded_block = bincode::serialize(&block).unwrap();
        let block_key = format!("{}{}", BLOCK_PREFIX, block.index);
        self.db.insert(block_key, encoded_block).unwrap();
        self.db.flush().unwrap();

        for tx in block.transactions {
            if tx.from != NETWORK_ADDRESS {
                let balance = self.load_balance(&tx.from);
                self.save_balance(&tx.from, balance - tx.amount);
            }

            let to_balance = self.load_balance(&tx.to);
            self.save_balance(&tx.to, to_balance + tx.amount);
        }
        true
    }

    pub fn is_valid(&self) -> bool {
        let chain = self.load_blockchain();
        for i in 1..chain.len() {
            let current = &chain[i];
            let previous = &chain[i - 1];

            if current.prev_hash != previous.hash {
                return false;
            }
            if !current.hash.starts_with(&"0".repeat(self.difficulty)) {
                return false;
            }
        }
        true
    }
}

impl Blockchain {
    pub fn load_balance(&self, address: &str) -> f64 {
        let key = format!("{}{}", BALANCE_PREFIX, address);
        if let Ok(Some(b)) = self.db.get(key) {
            let balance: f64 = bincode::deserialize(&b).unwrap();
            balance
        } else {
            0.0
        }
    }

    fn save_balance(&self, address: &str, balance: f64) {
        let key = format!("{}{}", BALANCE_PREFIX, address);
        let bytes_balance = bincode::serialize(&balance).unwrap();
        self.db.insert(key, bytes_balance).unwrap();
    }

    pub fn load_balances(&self) -> Vec<(String, f64)> {
        let mut balances = vec![];
        for key in self.db.scan_prefix(BALANCE_PREFIX) {
            let (key, val) = key.unwrap();

            let key_str = String::from_utf8(key.to_vec()).unwrap();
            let address = key_str.strip_prefix(BALANCE_PREFIX).unwrap().to_string();
            let balance: f64 = bincode::deserialize(&val).unwrap();
            balances.push((address, balance));
        }
        balances
    }
}
