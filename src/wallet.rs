use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::{OsRng, RngCore};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};

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

    pub fn save_to_file_encrypted(&self, filename: &str, password: &str) {
        let private_key_bytes = self.private_key.to_bytes();

        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));
        let cipher_text = cipher.encrypt(GenericArray::from_slice(&nonce), private_key_bytes.as_slice())
            .expect("Ошибка шифрования");

        let mut file = File::create(filename).expect("Не удалось создать файл");
        writeln!(file, "{}", general_purpose::STANDARD.encode(&salt)).unwrap();
        writeln!(file, "{}", general_purpose::STANDARD.encode(&nonce)).unwrap();
        writeln!(file, "{}", general_purpose::STANDARD.encode(&cipher_text)).unwrap();

        println!("Кошелек зашифрован и записан в файл {}", filename);
    }

    pub fn load_from_file_encrypted(filename: &str, password: &str) -> Self {
        let file_content = read_to_string(filename).expect("Не удалось открыть файл");
        let mut lines = file_content.lines();

        let salt = general_purpose::STANDARD.decode(lines.next().unwrap()).unwrap();
        let nonce = general_purpose::STANDARD.decode(lines.next().unwrap()).unwrap();
        let cipher_text = general_purpose::STANDARD.decode(lines.next().unwrap()).unwrap();

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));
        let decoded_key = cipher.decrypt(GenericArray::from_slice(&nonce), &cipher_text[..])
            .expect("Ошибка шифрования");

        let signing_key = SigningKey::try_from(decoded_key.as_slice()).expect("Некорректный ключ");
        let verifying_key = signing_key.verifying_key().clone();

        Wallet {
            private_key: signing_key,
            public_key: verifying_key,
        }
    }

}