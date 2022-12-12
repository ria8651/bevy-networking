use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, Client, ConnectionConfiguration, ConnectionEvent,
        QuinnetClientPlugin,
    },
    shared::ClientId,
};
use common::{ClientMessage, ServerMessage};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Users::default())
            .add_plugin(QuinnetClientPlugin::default())
            .add_startup_system(start_connection)
            .add_system(handle_client_events)
            .add_system(handle_server_messages);
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    character_entities: HashMap<ClientId, Entity>,
}

fn start_connection(mut client: ResMut<Client>) {
    client.open_connection(
        ConnectionConfiguration::new("127.0.0.1".to_string(), 6000, "0.0.0.0".to_string(), 0),
        CertificateVerificationMode::SkipVerification,
    );
}

fn handle_client_events(connection_events: EventReader<ConnectionEvent>, client: ResMut<Client>) {
    if !connection_events.is_empty() {
        // We are connected
        let username: String = "brian".to_string();

        println!("--- Joining with name: {}", username);
        println!("--- Type 'quit' to disconnect");

        client
            .connection()
            .send_message(ClientMessage::Join { name: username })
            .unwrap();

        connection_events.clear();
    }
}

fn handle_server_messages(mut users: ResMut<Users>, mut client: ResMut<Client>) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::ClientConnected {
                client_id,
                username,
            } => {
                // info!("{} joined", username);
                // users.names.insert(client_id, username);
            }
            ServerMessage::ClientDisconnected { client_id } => {
                // if let Some(username) = users.names.remove(&client_id) {
                //     println!("{} left", username);
                // } else {
                //     warn!("ClientDisconnected for an unknown client_id: {}", client_id)
                // }
            }
            ServerMessage::ChatMessage { client_id, message } => {
                // if let Some(username) = users.names.get(&client_id) {
                //     if client_id != users.self_id {
                //         println!("{}: {}", username, message);
                //     }
                // } else {
                //     warn!("Chat message from an unknown client_id: {}", client_id)
                // }
            }
            ServerMessage::Image { client_id, image } => {
                // if let Some(username) = users.names.get(&client_id) {
                //     if client_id != users.self_id {
                //         println!("{} sent an image: ", username);

                //         let tones = " .:-=+*#%@"; // 10 levels of gray
                //         for line in image.iter() {
                //             for pixel in line.iter() {
                //                 print!("{}", tones.chars().nth(*pixel as usize / 25).unwrap());
                //             }
                //             println!();
                //         }
                //     }
                // } else {
                //     warn!("Image from an unknown client_id: {}", client_id)
                // }
            }
            ServerMessage::InitClient {
                client_id,
                usernames,
            } => {
                // users.self_id = client_id;
                // users.names = usernames;
            }
        }
    }
}
