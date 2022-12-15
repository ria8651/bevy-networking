use bevy::{app::AppExit, prelude::*, window::exit_on_all_closed};
use bevy_networking::{
    client_connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput,
    ServerChannel, ServerMessages, PROTOCOL_ID,
};
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetError},
    run_if_client_connected, RenetClientPlugin,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin::default())
            .add_event::<PlayerCommand>()
            // .insert_resource(new_renet_client())
            .insert_resource(ClientLobby::default())
            .insert_resource(PlayerInput::default())
            .insert_resource(NetworkMapping::default())
            .add_system(player_input)
            .add_system(client_send_input.with_run_criteria(run_if_client_connected))
            .add_system(client_send_player_commands.with_run_criteria(run_if_client_connected))
            .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
            // .add_system_to_stage(
            //     CoreStage::PostUpdate,
            //     disconnect_on_exit.after(exit_on_all_closed),
            // )
            .add_system(panic_on_error_system);
    }
}

#[derive(Component)]
struct ControlledPlayer;

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Debug)]
struct PlayerInfo {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<u64, PlayerInfo>,
}

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.forward = keyboard_input.pressed(KeyCode::W);
    player_input.back = keyboard_input.pressed(KeyCode::S);
    player_input.right = keyboard_input.pressed(KeyCode::D);
    player_input.left = keyboard_input.pressed(KeyCode::A);
    player_input.up = keyboard_input.pressed(KeyCode::Space);
    player_input.down = keyboard_input.pressed(KeyCode::LShift);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(ClientChannel::Input, input_message);
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let client_id = client.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
            } => {
                println!("Player {} connected.", id);
                let mut client_entity = commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(translation[0], translation[1], translation[2]),
                    ..Default::default()
                });

                if client_id == id {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let transform = Transform {
                    translation,
                    ..Default::default()
                };
                commands.entity(*entity).insert(transform);
            }
        }
    }
}

fn disconnect_on_exit(exit: EventReader<AppExit>, mut client: ResMut<RenetClient>) {
    if !exit.is_empty() && client.is_connected() {
        client.disconnect();
    }
}

// use bevy::{prelude::*, utils::HashMap};
// use bevy_quinnet::{
//     client::{
//         certificate::CertificateVerificationMode, Client, ConnectionConfiguration, ConnectionEvent,
//         QuinnetClientPlugin,
//     },
//     shared::ClientId,
// };
// use bevy_voxel_engine::Velocity;
// use common::{ClientData, ClientMessage, ServerMessage};

// use crate::character::CharacterEntity;

// pub struct NetworkingPlugin;

// impl Plugin for NetworkingPlugin {
//     fn build(&self, app: &mut App) {
//         app.insert_resource(Users::default())
//             .add_plugin(QuinnetClientPlugin::default())
//             .add_system(handle_client_events)
//             .add_event::<CreateConnectionEvent>()
//             .add_event::<DisconnectEvent>()
//             .add_system(send_character_position)
//             .add_system(events_system)
//             .add_system(handle_server_messages);
//     }
// }

// pub struct CreateConnectionEvent {
//     pub ip: String,
// }
// pub struct DisconnectEvent;

// #[derive(Resource, Debug, Clone, Default)]
// struct Users {
//     self_id: ClientId,
//     client_data: HashMap<ClientId, ClientData>,
//     character_entities: HashMap<ClientId, Entity>,
// }

// #[derive(Component)]
// struct RemotePlayer;

// fn events_system(
//     mut create_connection_events: EventReader<CreateConnectionEvent>,
//     mut disconnect_events: EventReader<DisconnectEvent>,
//     mut client: ResMut<Client>,
// ) {
//     for event in create_connection_events.iter() {
//         client.open_connection(
//             ConnectionConfiguration::new(event.ip.clone(), 6000, "0.0.0.0".to_string(), 0),
//             CertificateVerificationMode::SkipVerification,
//         );
//     }

//     for _ in disconnect_events.iter() {
//         if client.connections().len() > 0 {
//             client
//                 .connection()
//                 .send_message(ClientMessage::Disconnect {})
//                 .unwrap();
//         }
//     }
// }

// fn send_character_position(
//     client: Res<Client>,
//     character_query: Query<(&Transform, &Velocity), With<CharacterEntity>>,
// ) {
//     if client.connections().len() == 0 {
//         return;
//     }

//     let (transform, velocity) = character_query.single();
//     client
//         .connection()
//         .send_message(ClientMessage::UpdatePosition {
//             position: transform.translation,
//             velocity: velocity.velocity,
//         })
//         .unwrap();
// }

// fn handle_client_events(connection_events: EventReader<ConnectionEvent>, client: ResMut<Client>) {
//     if !connection_events.is_empty() {
//         // We are connected
//         let username: String = "brian".to_string();

//         println!("--- Joining with name: {}", username);
//         println!("--- Type 'quit' to disconnect");

//         client
//             .connection()
//             .send_message(ClientMessage::Join {
//                 client_data: ClientData { username },
//             })
//             .unwrap();

//         connection_events.clear();
//     }
// }

// fn handle_server_messages(
//     mut commands: Commands,
//     mut users: ResMut<Users>,
//     mut client: ResMut<Client>,
//     mut character_query: Query<&mut Transform, With<RemotePlayer>>,
// ) {
//     if client.connections().len() == 0 {
//         return;
//     }

//     while let Some(message) = client
//         .connection_mut()
//         .try_receive_message::<ServerMessage>()
//     {
//         match message {
//             ServerMessage::ClientConnected {
//                 client_id,
//                 client_data,
//             } => {
//                 info!("{} joined", client_data.username);
//                 users.client_data.insert(client_id, client_data);
//             }
//             ServerMessage::ClientDisconnected { client_id } => {
//                 if let Some(entity) = users.character_entities.remove(&client_id) {
//                     commands.entity(entity).despawn();
//                 } else {
//                     warn!("ClientDisconnected for an unknown client_id: {}", client_id)
//                 }
//             }
//             ServerMessage::InitClient {
//                 client_id,
//                 client_data,
//             } => {
//                 users.self_id = client_id;
//                 users.client_data = client_data;
//             }
//             ServerMessage::UpdatePosition {
//                 client_id,
//                 position,
//                 ..
//             } => {
//                 if client_id != users.self_id {
//                     if let Some(entity) = users.character_entities.get(&client_id) {
//                         let mut transform = character_query
//                             .get_mut(*entity)
//                             .expect("Character entity destroyed");
//                         transform.translation = position;
//                     } else {
//                         users.character_entities.insert(
//                             client_id,
//                             commands
//                                 .spawn((
//                                     Transform::from_translation(position),
//                                     bevy_voxel_engine::Box {
//                                         half_size: IVec3::new(2, 4, 2),
//                                         material: 10,
//                                     },
//                                     RemotePlayer,
//                                 ))
//                                 .id(),
//                         );
//                     }
//                 }
//             }
//         }
//     }
// }
