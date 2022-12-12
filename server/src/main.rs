use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfigurationData,
    },
    shared::ClientId,
};
use common::{ClientMessage, ServerMessage};
use std::collections::HashMap;

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    names: HashMap<ClientId, String>,
}

fn handle_client_messages(mut server: ResMut<Server>, mut users: ResMut<Users>) {
    let endpoint = server.endpoint_mut();
    while let Ok(Some((message, client_id))) = endpoint.receive_message::<ClientMessage>() {
        match message {
            ClientMessage::Join { name } => {
                if users.names.contains_key(&client_id) {
                    warn!(
                        "Received a Join from an already connected client: {}",
                        client_id
                    )
                } else {
                    info!("{} connected", name);
                    users.names.insert(client_id, name.clone());
                    // Initialize this client with existing state
                    endpoint
                        .send_message(
                            client_id,
                            ServerMessage::InitClient {
                                client_id: client_id,
                                usernames: users.names.clone(),
                            },
                        )
                        .unwrap();
                    // Broadcast the connection event
                    endpoint
                        .send_group_message(
                            users.names.keys().into_iter(),
                            ServerMessage::ClientConnected {
                                client_id: client_id,
                                username: name,
                            },
                        )
                        .unwrap();
                }
            }
            ClientMessage::Disconnect {} => {
                // We tell the server to disconnect this user
                endpoint.disconnect_client(client_id).unwrap();
                handle_disconnect(endpoint, &mut users, client_id);
            }
            ClientMessage::ChatMessage { message } => {
                info!(
                    "Chat message | {:?}: {}",
                    users.names.get(&client_id),
                    message
                );
                endpoint.try_send_group_message(
                    users.names.keys().into_iter(),
                    ServerMessage::ChatMessage {
                        client_id: client_id,
                        message: message,
                    },
                );
            }
            ClientMessage::Image { image } => {
                info!("Image | {:?}", users.names.get(&client_id));
                endpoint.try_send_group_message(
                    users.names.keys().into_iter(),
                    ServerMessage::Image {
                        client_id: client_id,
                        image: image,
                    },
                );
            }
        }
    }
}

fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut users: ResMut<Users>,
) {
    // The server signals us about users that lost connection
    for client in connection_lost_events.iter() {
        handle_disconnect(server.endpoint_mut(), &mut users, client.id);
    }
}

/// Shared disconnection behaviour, whether the client lost connection or asked to disconnect
fn handle_disconnect(endpoint: &mut Endpoint, users: &mut ResMut<Users>, client_id: ClientId) {
    // Remove this user
    if let Some(username) = users.names.remove(&client_id) {
        // Broadcast its deconnection

        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ClientDisconnected {
                    client_id: client_id,
                },
            )
            .unwrap();
        info!("{} disconnected", username);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        )
    }
}

fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfigurationData::new("127.0.0.1".to_string(), 6000, "0.0.0.0".to_string()),
            CertificateRetrievalMode::GenerateSelfSigned,
        )
        .unwrap();
}

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(QuinnetServerPlugin::default())
        .insert_resource(Users::default())
        .add_startup_system(start_listening)
        .add_system(handle_client_messages)
        .add_system(handle_server_events)
        .run();
}
