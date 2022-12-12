use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Messages from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    Disconnect {},
    ChatMessage { message: String },
    Image { image: Vec<Vec<u8>> },
}

// Messages from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    ClientConnected {
        client_id: ClientId,
        username: String,
    },
    ClientDisconnected {
        client_id: ClientId,
    },
    ChatMessage {
        client_id: ClientId,
        message: String,
    },
    Image {
        client_id: ClientId,
        image: Vec<Vec<u8>>,
    },
    InitClient {
        client_id: ClientId,
        usernames: HashMap<ClientId, String>,
    },
}
