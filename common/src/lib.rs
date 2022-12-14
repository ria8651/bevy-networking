use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

// Data about a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientData {
    pub username: String,
}

// Messages from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { client_data: ClientData },
    Disconnect {},
    UpdatePosition { position: Vec3, velocity: Vec3 },
}

// Messages from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    ClientConnected {
        client_id: ClientId,
        client_data: ClientData,
    },
    ClientDisconnected {
        client_id: ClientId,
    },
    InitClient {
        client_id: ClientId,
        client_data: HashMap<ClientId, ClientData>,
    },
    UpdatePosition {
        client_id: ClientId,
        position: Vec3,
        velocity: Vec3,
    },
}
