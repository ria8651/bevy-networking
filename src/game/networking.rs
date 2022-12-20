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
    SpawnNetworkedEntity {
        client_id: u64,
        entity: Entity,
        entity_type: NetworkedEntityType,
        position: Vec3,
        velocity: Vec3,
    },
    UpdateNetworkedEntity {
        client_id: u64,
        entity: Entity,
        position: Vec3,
        velocity: Vec3,
    },
    DespawnNetworkedEntity {
        client_id: u64,
        entity: Entity,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessages {
    ChatMessage {
        message: String,
    },
    UpdatePlayer {
        position: Vec3,
        velocity: Vec3,
    },
    SpawnNetworkedEntity {
        entity: Entity,
        entity_type: NetworkedEntityType,
        position: Vec3,
        velocity: Vec3,
    },
    UpdateNetworkedEntity {
        entity: Entity,
        position: Vec3,
        velocity: Vec3,
    },
    DespawnNetworkedEntity {
        entity: Entity,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum NetworkedEntityType {
    Bullet(u32),
}
