use super::{
    character::CharacterEntity,
    networking::{ClientMessages, NetworkedEntityType, ServerMessages},
};
use crate::{game::InGame, GameState};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_voxel_engine::{Particle, Velocity};
use rand::Rng;
use renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig};
use std::{
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
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
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(update_player)
                    .with_system(update_networked_entitys),
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
    pub players: HashMap<u64, ClientPlayerData>,
    // maps remote entity id to local entity for each player
    pub networked_entitys: HashMap<u64, HashMap<Entity, Entity>>,
    pub local_networked_entitys: HashSet<Entity>,
}

pub struct ClientPlayerData {
    pub username: String,
    pub entity: Entity,
}

#[derive(Component)]
struct RemotePlayer;

#[derive(Component)]
pub struct LocalNetworkedEntity {
    pub entity_type: NetworkedEntityType,
}

#[derive(Component)]
pub struct RemoteNetworkedEntity {
    pub velocity: Vec3,
}

impl Client {
    pub fn new(ip: String, _: String) -> Self {
        let client_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server_addr = ip.to_socket_addrs().unwrap().next().unwrap();
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
            networked_entitys: HashMap::default(),
            local_networked_entitys: HashSet::default(),
        }
    }
}

fn process_server_messages(
    mut commands: Commands,
    mut client_resource: ResMut<ClientResource>,
    mut network_players: Query<&mut Transform, With<RemotePlayer>>,
    mut networked_entitys: Query<
        (&mut RemoteNetworkedEntity, &mut Transform),
        Without<RemotePlayer>,
    >,
) {
    if let Some(client) = (*client_resource).as_mut() {
        while let Some(message) = client.client.receive_message(DefaultChannel::Reliable) {
            let message: ServerMessages = bincode::deserialize(&message).unwrap();
            match message {
                ServerMessages::ClientConnected {
                    client_id,
                    username,
                } => {
                    let entity = commands
                        .spawn((
                            Transform::default(),
                            bevy_voxel_engine::Box {
                                half_size: IVec3::new(2, 4, 2),
                                material: 10,
                            },
                            RemotePlayer,
                            InGame,
                        ))
                        .id();

                    client.players.insert(
                        client_id,
                        ClientPlayerData {
                            username: username.clone(),
                            entity,
                        },
                    );
                    info!("Player {} ({}) connected.", username, client_id);
                }
                ServerMessages::ClientDisconnected { client_id } => {
                    let client_player_data = client.players.remove(&client_id).unwrap();
                    commands.entity(client_player_data.entity).despawn();
                    info!(
                        "Player {} ({}) disconnected.",
                        client_player_data.username, client_id
                    );
                }
                ServerMessages::ChatMessage { client_id, message } => {
                    let username = &client.players.get(&client_id).unwrap().username;
                    info!("{}: {}", username, message);
                }
                ServerMessages::UpdatePlayer {
                    client_id,
                    position,
                    ..
                } => {
                    if let Some(player) = client.players.get_mut(&client_id) {
                        if let Ok(mut transform) = network_players.get_mut(player.entity) {
                            transform.translation = position;
                        }
                    }
                }
                ServerMessages::SpawnNetworkedEntity {
                    client_id,
                    entity,
                    entity_type,
                    position,
                    velocity,
                } => match entity_type {
                    NetworkedEntityType::Bullet(bullet_type) => {
                        let material = match bullet_type {
                            1 => 120,
                            2 => 121,
                            _ => 10,
                        };

                        let local_entity = commands
                            .spawn((
                                Transform::from_translation(position),
                                Particle { material },
                                RemoteNetworkedEntity { velocity },
                                InGame,
                            ))
                            .id();

                        client
                            .networked_entitys
                            .entry(client_id)
                            .or_default()
                            .insert(entity, local_entity);
                    }
                },
                ServerMessages::UpdateNetworkedEntity {
                    client_id,
                    entity,
                    position,
                    velocity,
                } => {
                    let local_entity = client.networked_entitys[&client_id][&entity];
                    if let Ok(query) = networked_entitys.get_mut(local_entity) {
                        let (mut remote_networked_entity, mut transform) = query;
                        transform.translation = position;
                        remote_networked_entity.velocity = velocity;
                    }
                }
                ServerMessages::DespawnNetworkedEntity { client_id, entity } => {
                    let local_entity = client.networked_entitys[&client_id][&entity];
                    commands.entity(local_entity).despawn();
                    client
                        .networked_entitys
                        .get_mut(&client_id)
                        .unwrap()
                        .remove(&entity);
                }
            }
        }
    }
}

fn update_player(
    mut client_resource: ResMut<ClientResource>,
    player: Query<(&Transform, &Velocity), With<CharacterEntity>>,
) {
    if let Some(client) = (*client_resource).as_mut() {
        let (player, velocity) = player.single();
        let message = ClientMessages::UpdatePlayer {
            position: player.translation,
            velocity: velocity.velocity,
        };
        client.client.send_message(
            DefaultChannel::Reliable,
            bincode::serialize(&message).unwrap(),
        );
    }
}

fn update_networked_entitys(
    mut client_resource: ResMut<ClientResource>,
    networked_entitys: Query<(Entity, &LocalNetworkedEntity, &Transform, Option<&Velocity>)>,
) {
    if let Some(client) = (*client_resource).as_mut() {
        for (entity, local_networked_entity, transform, velocity) in networked_entitys.iter() {
            if client.local_networked_entitys.contains(&entity) {
                client.client.send_message(
                    DefaultChannel::Reliable,
                    bincode::serialize(&ClientMessages::UpdateNetworkedEntity {
                        entity,
                        position: transform.translation,
                        velocity: velocity.map(|v| v.velocity).unwrap_or_default(),
                    })
                    .unwrap(),
                );
            } else {
                client.client.send_message(
                    DefaultChannel::Reliable,
                    bincode::serialize(&ClientMessages::SpawnNetworkedEntity {
                        entity,
                        entity_type: local_networked_entity.entity_type,
                        position: transform.translation,
                        velocity: velocity.map(|v| v.velocity).unwrap_or_default(),
                    })
                    .unwrap(),
                );
                client.local_networked_entitys.insert(entity);
            }
        }

        for entity in client.local_networked_entitys.clone().iter() {
            if networked_entitys.get(*entity).is_err() {
                client.client.send_message(
                    DefaultChannel::Reliable,
                    bincode::serialize(&ClientMessages::DespawnNetworkedEntity { entity: *entity })
                        .unwrap(),
                );
                client.local_networked_entitys.remove(entity);
            }
        }
    }
}
