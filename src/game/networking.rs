use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessages {
    ClientConnected { client_id: u64, username: String },
    ClientDisconnected { client_id: u64 },
    ChatMessage { client_id: u64, message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessages {
    ChatMessage(String),
}
