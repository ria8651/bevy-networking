use super::networking::ServerMessages;
use crate::GameState;
use bevy::{prelude::*, utils::HashMap};
use rand::Rng;
use renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientResource(None))
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::on_update(GameState::Game).with_system(update),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(GameState::Game).with_system(send_packets),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game).with_system(process_server_messages),
            )
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(disconnect));
    }
}

fn update(mut client_resource: ResMut<ClientResource>, time: Res<Time>) {
    if let Some(client) = (*client_resource).as_mut() {
        if let Err(e) = client.client.update(time.delta()) {
            error!("{}", e);
        }
    }
}

fn send_packets(mut client_resource: ResMut<ClientResource>) {
    if let Some(client) = (*client_resource).as_mut() {
        client.client.send_packets().unwrap();
    }
}

fn disconnect(mut client_resource: ResMut<ClientResource>) {
    if let Some(client) = (*client_resource).as_mut() {
        client.client.disconnect();
        **client_resource = None;
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ClientResource(pub Option<Client>);

pub struct Client {
    pub client: RenetClient,
    pub players: HashMap<u64, String>,
}

impl Client {
    pub fn new(ip: String, _: String) -> Self {
        let client_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server_addr = ip.parse().unwrap();
        let socket = UdpSocket::bind(client_addr).unwrap();
        let connection_config = RenetConnectionConfig::default();

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let mut rng = rand::thread_rng();
        let authentication = ClientAuthentication::Unsecure {
            protocol_id: 0,
            client_id: rng.gen::<u64>(),
            server_addr,
            user_data: None,
        };

        info!("Client connected to {}", server_addr);

        Self {
            client: RenetClient::new(current_time, socket, connection_config, authentication)
                .unwrap(),
            players: HashMap::default(),
        }
    }
}

fn process_server_messages(mut client_resource: ResMut<ClientResource>) {
    if let Some(client) = (*client_resource).as_mut() {
        while let Some(message) = client.client.receive_message(DefaultChannel::Reliable) {
            let message: ServerMessages = bincode::deserialize(&message).unwrap();
            match message {
                ServerMessages::ClientConnected {
                    client_id,
                    username,
                } => {
                    client.players.insert(client_id, username.clone());
                    info!("Player {} ({}) connected.", username, client_id);
                }
                ServerMessages::ClientDisconnected { client_id } => {
                    let username = client.players.remove(&client_id).unwrap();
                    info!("Player {} ({}) disconnected.", username, client_id);
                }
                ServerMessages::ChatMessage { client_id, message } => {
                    let username = client.players.get(&client_id).unwrap();
                    info!("{}: {}", username, message);
                }
            }
        }
    }
}
