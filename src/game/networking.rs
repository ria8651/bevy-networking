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
        transform: NetworkTransform,
    },
    UpdateNetworkedEntity {
        client_id: u64,
        entity: Entity,
        transform: NetworkTransform,
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
        transform: NetworkTransform,
    },
    UpdateNetworkedEntity {
        entity: Entity,
        transform: NetworkTransform,
    },
    DespawnNetworkedEntity {
        entity: Entity,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum NetworkedEntityType {
    Bullet(u32),
    Portal(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct NetworkTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub velocity: Vec3,
}

impl NetworkTransform {
    pub fn from_transform(transform: &Transform, velocity: Vec3) -> Self {
        Self {
            position: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
            velocity,
        }
    }
}

impl From<&NetworkTransform> for Transform {
    fn from(network_transform: &NetworkTransform) -> Self {
        Self {
            translation: network_transform.position,
            rotation: network_transform.rotation,
            scale: network_transform.scale,
        }
    }
}