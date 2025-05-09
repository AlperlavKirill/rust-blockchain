use std::fs;
use std::io::{Read, Write};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::ecdsa::signature::{Signer, Verifier};
use k256::elliptic_curve::rand_core::OsRng;

pub struct Wallet {
    pub public_key: VerifyingKey,
    pub private_key: SigningKey,
}

impl Wallet {
    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key().clone();
        Wallet {private_key, public_key}
    }

    pub fn address(&self) -> String {
        hex::encode(self.public_key.to_sec1_bytes())
    }

    pub fn sign(&self, transaction_data: &[u8]) -> Vec<u8> {
        let signature: Signature = self.private_key.sign(transaction_data);
        signature.to_vec()
    }

    pub fn verify(public_key: &[u8], transaction_data: &[u8], signature: &[u8]) -> bool {
        let verifying_key = VerifyingKey::from_sec1_bytes(public_key).unwrap();
        let signature = Signature::try_from(signature).unwrap();
        verifying_key.verify(transaction_data, &signature).is_ok()
    }

    pub fn save_to_file(&self, filename: &str) {
        let hex_str = hex::encode(self.private_key.to_bytes());

        let mut file = fs::File::create(filename).expect("Не удалось создать файл кошелька");
        file.write_all(hex_str.as_bytes()).expect("Не удалось записать данные кошелька в файл");
        println!("Кошелек сохранен в {}", filename)
    }

    pub fn load_from_file(filename: &str) -> Self {
        let mut file = fs::File::open(filename).expect("Не удалось открыть файл кошелька");
        let mut hex_str = String::new();
        file.read_to_string(&mut hex_str).expect("Не удалось прочитать файл");

        let private_key_bytes = hex::decode(hex_str).expect("Некорректный формат данных");
        let private_key = SigningKey::try_from(private_key_bytes.as_slice()).expect("Некорректный ключ");
        let public_key = private_key.verifying_key().clone();

        println!("Кошелек загружен из {}", filename);

        Wallet {private_key, public_key}
    }

}