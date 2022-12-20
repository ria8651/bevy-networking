use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessages {
    ClientConnected {
        client_id: u64,
        username: String,
    },
    ClientDisconnected {
        client_id: u64,
    },
    ChatMessage {
        client_id: u64,
        message: String,
    },
    UpdatePlayer {
        client_id: u64,
        position: Vec3,
        velocity: Vec3,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessages {
    ChatMessage { message: String },
    UpdatePlayer { position: Vec3, velocity: Vec3 },
}
