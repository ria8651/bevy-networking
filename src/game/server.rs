use crate::{
    game::networking::{ClientMessages, ServerMessages},
    GameState,
};
use bevy::{prelude::*, utils::HashMap};
use renet::{
    DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
    ServerEvent,
};
use std::{
    net::{ToSocketAddrs, UdpSocket},
    time::SystemTime,
};

use super::networking::{NetworkTransform, NetworkedEntityType};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerResource(None))
            .add_event::<ServerEvent>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::on_update(GameState::Game).with_system(update),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(GameState::Game).with_system(send_packets),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(process_server_events)
                    .with_system(process_client_messages.after(process_server_events)),
            )
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(close_server));
    }
}

fn update(
    mut server_resource: ResMut<ServerResource>,
    time: Res<Time>,
    mut server_events: EventWriter<ServerEvent>,
) {
    if let Some(server) = (*server_resource).as_mut() {
        if let Err(e) = server.server.update(time.delta()) {
            error!("{}", e);
        }

        while let Some(event) = server.server.get_event() {
            server_events.send(event);
        }
    }
}

fn send_packets(mut server_resource: ResMut<ServerResource>) {
    if let Some(server) = (*server_resource).as_mut() {
        server.server.send_packets().unwrap();
    }
}

fn close_server(mut server_resource: ResMut<ServerResource>) {
    if let Some(server) = (*server_resource).as_mut() {
        let clients = server.server.clients_id();
        for client in clients {
            server.server.disconnect(client);
        }

        **server_resource = None;
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ServerResource(pub Option<Server>);

pub struct Server {
    pub server: RenetServer,
    pub players: HashMap<u64, String>,
    networked_entities: HashMap<u64, HashMap<Entity, NetworkedEntity>>,
}

struct NetworkedEntity {
    entity_type: NetworkedEntityType,
    transform: NetworkTransform,
}

impl Server {
    pub fn new(bind_ip: String, _: String) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:1234").unwrap();
        let server_addr = bind_ip.to_socket_addrs().unwrap().next().unwrap();
        let connection_config = RenetConnectionConfig::default();
        let server_config = ServerConfig::new(64, 0, server_addr, ServerAuthentication::Unsecure);

        // let register_server = RegisterServer {
        //     name: lobby_name,
        //     address: server_addr,
        //     max_clients: server_config.max_clients as u64,
        //     private_key,
        //     password,
        //     current_clients: 0,
        // };
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        info!("Server started on {}", server_addr);

        Self {
            server: RenetServer::new(current_time, server_config, connection_config, socket)
                .unwrap(),
            players: HashMap::default(),
            networked_entities: HashMap::default(),
        }
    }
}

fn process_server_events(
    mut server_resource: ResMut<ServerResource>,
    mut server_events: EventReader<ServerEvent>,
) {
    if let Some(server) = (*server_resource).as_mut() {
        for event in server_events.iter() {
            match event {
                ServerEvent::ClientConnected(id, _) => {
                    let username = "Bob".to_string();
                    server.server.broadcast_message_except(
                        *id,
                        DefaultChannel::Reliable,
                        bincode::serialize(&ServerMessages::ClientConnected {
                            client_id: *id,
                            username: username.clone(),
                        })
                        .unwrap(),
                    );

                    // send currently connected players to the new player
                    for &player_id in server.players.keys() {
                        server.server.send_message(
                            *id,
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::ClientConnected {
                                client_id: player_id,
                                username: username.clone(),
                            })
                            .unwrap(),
                        );
                    }

                    // send currently spawned entities to the new player
                    for (client_id, entities) in server.networked_entities.iter() {
                        for (entity, networked_entity) in entities.iter() {
                            info!("Sending entity to client: {:?}", entity);
                            server.server.send_message(
                                *id,
                                DefaultChannel::Reliable,
                                bincode::serialize(&ServerMessages::SpawnNetworkedEntity {
                                    client_id: *client_id,
                                    entity: *entity,
                                    entity_type: networked_entity.entity_type,
                                    transform: networked_entity.transform,
                                })
                                .unwrap(),
                            );
                        }
                    }

                    server.players.insert(*id, username.clone());

                    info!("Player {} ({}) connected.", username.clone(), id);
                }
                ServerEvent::ClientDisconnected(id) => {
                    let username = server.players.remove(id).unwrap();

                    server.server.broadcast_message(
                        DefaultChannel::Reliable,
                        bincode::serialize(&ServerMessages::ClientDisconnected { client_id: *id })
                            .unwrap(),
                    );

                    info!("Player {} ({}) disconnected.", username, id);
                }
            }
        }
    }
}

fn process_client_messages(mut server_resource: ResMut<ServerResource>) {
    if let Some(server) = (*server_resource).as_mut() {
        for client_id in server.server.clients_id().into_iter() {
            while let Some(message) = server
                .server
                .receive_message(client_id, DefaultChannel::Reliable)
            {
                let message: ClientMessages = bincode::deserialize(&message).unwrap();
                match message {
                    ClientMessages::ChatMessage { message } => {
                        info!("{}: {}", server.players[&client_id], message);
                        server.server.broadcast_message(
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::ChatMessage { client_id, message })
                                .unwrap(),
                        );
                    }
                    ClientMessages::UpdatePlayer { position, velocity } => {
                        server.server.broadcast_message_except(
                            client_id,
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::UpdatePlayer {
                                client_id,
                                position,
                                velocity,
                            })
                            .unwrap(),
                        );
                    }
                    ClientMessages::SpawnNetworkedEntity {
                        entity,
                        entity_type,
                        transform,
                    } => {
                        server.server.broadcast_message_except(
                            client_id,
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::SpawnNetworkedEntity {
                                client_id,
                                entity,
                                entity_type,
                                transform,
                            })
                            .unwrap(),
                        );

                        let networked_entity = NetworkedEntity {
                            entity_type,
                            transform,
                        };

                        server
                            .networked_entities
                            .entry(client_id)
                            .or_default()
                            .insert(entity, networked_entity);
                    }
                    ClientMessages::UpdateNetworkedEntity { entity, transform } => {
                        server.server.broadcast_message_except(
                            client_id,
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::UpdateNetworkedEntity {
                                client_id,
                                entity,
                                transform,
                            })
                            .unwrap(),
                        );

                        server
                            .networked_entities
                            .get_mut(&client_id)
                            .unwrap()
                            .get_mut(&entity)
                            .unwrap()
                            .transform = transform;
                    }
                    ClientMessages::DespawnNetworkedEntity { entity } => {
                        server.server.broadcast_message_except(
                            client_id,
                            DefaultChannel::Reliable,
                            bincode::serialize(&ServerMessages::DespawnNetworkedEntity {
                                client_id,
                                entity,
                            })
                            .unwrap(),
                        );

                        server
                            .networked_entities
                            .get_mut(&client_id)
                            .unwrap()
                            .remove(&entity);
                    }
                }
            }
        }
    }
}
