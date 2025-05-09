// p2p - сеть узлов.
// узлы обмениваются транзакциями, блоками, списками известных узлов

// связь между узлами - tcp
// команды: ping, tx, block

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use crate::block::Block;
use crate::blockchain::Blockchain;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub command: String,
    pub payload: Vec<u8>,
}

#[derive(Clone)]
pub struct P2P {
    pub nodes: Vec<String>,
    pub db: String,
}

impl P2P {
    pub fn new(nodes: Vec<String>, db: String) -> Self {
        Self { nodes, db }
    }

    pub fn start_server(&self, addr: String) {
        let listener = TcpListener::bind(addr).expect("Не удалось запустить сервер");

        println!("Сервер успешно запущен");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Новое соединение: {}", stream.peer_addr().unwrap());
                    let db_path = self.db.clone();
                    thread::spawn(move || {
                        P2P::handle_connection(stream, db_path);
                    });
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
    }

    fn handle_connection(mut stream: TcpStream, db: String) {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).unwrap();

        if buffer.is_empty() {
            return;
        }

        let msg: Message = bincode::deserialize(&buffer).unwrap();

        println!("Получено сообщение {:?}", msg);

        match msg.command.as_str() {
            "ping" => {
                print!("Ping от другого узла");
            }
            "tx" => {
                println!("Получена транзакция");
            }
            "block" => {
                println!("Получен блок");
                let block: Block = bincode::deserialize(&msg.payload).unwrap();

                let mut blockchain = Blockchain::new(&db, 3);
                if blockchain.add_block_from_p2p(block) {
                    println!("Блок успешно добавлен в локальный блокчейн {}", db)
                } else {
                    println!("Блок невалидный");
                }
            }
            _ => {
                println!("Неизвестная команда")
            }
        }
    }

    pub fn send_message(&self, node: &str, message: &Message) {
        if let Ok(mut stream) = TcpStream::connect(node) {
            let data = bincode::serialize(message).unwrap();
            stream.write_all(&data).unwrap();
            println!("Сообщение отправлено -> {}", node);
        } else {
            println!("Не удалось подключиться к {}", node)
        }
    }

    pub fn broadcast(&self, message: &Message) {
        for node in &self.nodes {
            self.send_message(node, message);
        }
    }
}
