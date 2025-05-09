mod block;
mod blockchain;
mod p2p;
mod transaction;
mod utils;
mod wallet;
mod api;

use crate::api::AppState;
use crate::p2p::P2P;
use blockchain::Blockchain;
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    db: String,
    #[arg(long)]
    p2p_port: u16,
    #[arg(long)]
    api_port: u16,
    #[arg(long)]
    nodes: String,
}

fn main() {
    let cli = Cli::parse();

    let db_name = cli.db;
    let blockchain = Arc::new(Mutex::new(Blockchain::new(&db_name, 3)));

    let nodes = cli.nodes
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let p2p = P2P::new(nodes, db_name);
    let p2p_port = cli.p2p_port;

    let p2p_server = p2p.clone();
    thread::spawn(move || {
        let bind_addr = format!("127.0.0.1:{}", p2p_port);
        p2p_server.start_server(bind_addr);
    });

    let p2p_api = p2p.clone();
    let api_port = cli.api_port;
    tokio::runtime::Runtime::new().unwrap().block_on(async move {
        api::start_api(AppState { blockchain, p2p: Arc::new(p2p_api)}, api_port).await;
    });
}
