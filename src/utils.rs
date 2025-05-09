use std::time::SystemTime;
use sha2::{Digest, Sha256};

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn calculate_hash(data_to_hash: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&data_to_hash);
    let result = hasher.finalize();
    hex::encode(result)
}