use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, Client, ConnectionConfiguration, ConnectionEvent,
        QuinnetClientPlugin,
    },
    shared::ClientId,
};
use bevy_voxel_engine::Velocity;
use common::{ClientData, ClientMessage, ServerMessage};

use crate::character::CharacterEntity;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Users::default())
            .add_plugin(QuinnetClientPlugin::default())
            .add_system(handle_client_events)
            .add_event::<CreateConnectionEvent>()
            .add_event::<DisconnectEvent>()
            .add_system(send_character_position)
            .add_system(events_system)
            .add_system(handle_server_messages);
    }
}

pub struct CreateConnectionEvent {
    pub ip: String,
}
pub struct DisconnectEvent;

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    client_data: HashMap<ClientId, ClientData>,
    character_entities: HashMap<ClientId, Entity>,
}

#[derive(Component)]
struct RemotePlayer;

fn events_system(
    mut create_connection_events: EventReader<CreateConnectionEvent>,
    mut disconnect_events: EventReader<DisconnectEvent>,
    mut client: ResMut<Client>,
) {
    for event in create_connection_events.iter() {
        client.open_connection(
            ConnectionConfiguration::new(event.ip.clone(), 6000, "0.0.0.0".to_string(), 0),
            CertificateVerificationMode::SkipVerification,
        );
    }

    for _ in disconnect_events.iter() {
        if client.connections().len() > 0 {
            client
                .connection()
                .send_message(ClientMessage::Disconnect {})
                .unwrap();
        }
    }
}

fn send_character_position(
    client: Res<Client>,
    character_query: Query<(&Transform, &Velocity), With<CharacterEntity>>,
) {
    if client.connections().len() == 0 {
        return;
    }

    let (transform, velocity) = character_query.single();
    client
        .connection()
        .send_message(ClientMessage::UpdatePosition {
            position: transform.translation,
            velocity: velocity.velocity,
        })
        .unwrap();
}

fn handle_client_events(connection_events: EventReader<ConnectionEvent>, client: ResMut<Client>) {
    if !connection_events.is_empty() {
        // We are connected
        let username: String = "brian".to_string();

        println!("--- Joining with name: {}", username);
        println!("--- Type 'quit' to disconnect");

        client
            .connection()
            .send_message(ClientMessage::Join {
                client_data: ClientData { username },
            })
            .unwrap();

        connection_events.clear();
    }
}

fn handle_server_messages(
    mut commands: Commands,
    mut users: ResMut<Users>,
    mut client: ResMut<Client>,
    mut character_query: Query<&mut Transform, With<RemotePlayer>>,
) {
    if client.connections().len() == 0 {
        return;
    }

    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::ClientConnected {
                client_id,
                client_data,
            } => {
                info!("{} joined", client_data.username);
                users.client_data.insert(client_id, client_data);
            }
            ServerMessage::ClientDisconnected { client_id } => {
                if let Some(entity) = users.character_entities.remove(&client_id) {
                    commands.entity(entity).despawn();
                } else {
                    warn!("ClientDisconnected for an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::InitClient {
                client_id,
                client_data,
            } => {
                users.self_id = client_id;
                users.client_data = client_data;
            }
            ServerMessage::UpdatePosition {
                client_id,
                position,
                ..
            } => {
                if client_id != users.self_id {
                    if let Some(entity) = users.character_entities.get(&client_id) {
                        let mut transform = character_query
                            .get_mut(*entity)
                            .expect("Character entity destroyed");
                        transform.translation = position;
                    } else {
                        users.character_entities.insert(
                            client_id,
                            commands
                                .spawn((
                                    Transform::from_translation(position),
                                    bevy_voxel_engine::Box {
                                        half_size: IVec3::new(2, 4, 2),
                                        material: 10,
                                    },
                                    RemotePlayer,
                                ))
                                .id(),
                        );
                    }
                }
            }
        }
    }
}
