use crate::blockchain::Blockchain;
use crate::p2p::{Message, P2P};
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::signal;

#[derive(Deserialize)]
pub struct NewTransactionData {
    pub from_file: String,
    pub to: String,
    pub amount: f64,
    pub password: String,
}

#[derive(Deserialize)]
pub struct WalletAccessData {
    pub file_name: String,
    pub password: String,
}

#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub p2p: Arc<P2P>
}

pub async fn start_api(state: AppState, port: u16) {
    let api = Router::new()
        .route("/balance/:address", get(get_balance))
        .route("/balances", get(get_balances))
        .route("/wallet", get(load_wallet))
        .route("/wallet/create", put(create_wallet))
        .route("/tx", post(create_tx))
        .route("/valid", get(valid_blockchain))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    println!("API Gateway запушен на http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, api.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

async fn get_balance(Path(address): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let blockchain =  state.blockchain.lock().unwrap();
    Json(blockchain.load_balance(&address))
}

async fn get_balances(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = state.blockchain.lock().unwrap();
    Json(blockchain.load_balances())
}

async fn load_wallet(Json(access_data): Json<WalletAccessData>) -> impl IntoResponse {
    let wallet = Wallet::load_from_file_encrypted(&access_data.file_name, &access_data.password);
    Json(format!("Адрес кошелька: {}", wallet.address()))
}

async fn create_wallet(Json(access_data): Json<WalletAccessData>) -> impl IntoResponse {
    let wallet = Wallet::new();
    wallet.save_to_file_encrypted(&access_data.file_name, &access_data.password);
    Json(format!("Адрес кошелька {}", wallet.address()))
}

async fn create_tx(State(state): State<AppState>, Json(tx): Json<NewTransactionData>) -> impl IntoResponse {
    let wallet = Wallet::load_from_file_encrypted(&tx.from_file, &tx.password);
    let from_address = wallet.address();

    let transaction_data = format!("{}{}{}", from_address, tx.to, tx.amount);
    let signature = wallet.sign(transaction_data.as_bytes());

    let new_tx = Transaction {
        from: from_address.clone(),
        to: tx.to.clone(),
        amount: tx.amount,
        signature,
        public_key: wallet.public_key.to_sec1_bytes().to_vec(),
    };

    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.add_block(&from_address, vec![new_tx]);

    let block = blockchain.latest_block().unwrap();

    let message = Message {
        command: "block".to_string(),
        payload: bincode::serialize(&block).unwrap()
    };

    state.p2p.broadcast(&message);

    Json("Транзакция успешно добавлена".to_string())
}

async fn valid_blockchain(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = state.blockchain.lock().unwrap();
    Json(blockchain.is_valid())
}

async fn shutdown_signal() {
    if let Err(e) = signal::ctrl_c().await {
        eprintln!("Не удалось обработать CTRL+C: {}", e);
    }
    println!("Получен Ctrl+C. Завершаем приложение.")
}
