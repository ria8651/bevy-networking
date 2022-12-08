use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    log::LogPlugin,
    prelude::{
        info, warn, App, Commands, CoreStage, Deref, DerefMut, EventReader, EventWriter, Res,
        ResMut, Resource,
    },
};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, Client, ConnectionConfiguration, ConnectionEvent,
        QuinnetClientPlugin,
    },
    shared::ClientId,
};
use protocol::{ClientMessage, ServerMessage};
use std::{collections::HashMap, thread, time::Duration};
use tokio::sync::mpsc;

mod protocol;

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(QuinnetClientPlugin::default())
        .insert_resource(Users::default())
        .add_startup_system(start_terminal_listener)
        .add_startup_system(start_connection)
        .add_system(handle_terminal_messages)
        .add_system(handle_server_messages)
        .add_system(handle_client_events)
        // CoreStage::PostUpdate so that AppExit events generated in the previous stage are available
        .add_system_to_stage(CoreStage::PostUpdate, on_app_exit)
        .run();
}

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

#[derive(Resource, Deref, DerefMut)]
struct TerminalReceiver(mpsc::Receiver<String>);

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Res<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        // TODO Clean: event to let the async client send its last messages.
        thread::sleep(Duration::from_secs_f32(0.1));
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
                info!("{} joined", username);
                users.names.insert(client_id, username);
            }
            ServerMessage::ClientDisconnected { client_id } => {
                if let Some(username) = users.names.remove(&client_id) {
                    println!("{} left", username);
                } else {
                    warn!("ClientDisconnected for an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::ChatMessage { client_id, message } => {
                if let Some(username) = users.names.get(&client_id) {
                    if client_id != users.self_id {
                        println!("{}: {}", username, message);
                    }
                } else {
                    warn!("Chat message from an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::Image { client_id, image } => {
                if let Some(username) = users.names.get(&client_id) {
                    if client_id != users.self_id {
                        println!("{} sent an image: ", username);

                        let tones = " .:-=+*#%@"; // 10 levels of gray
                        for line in image.iter() {
                            for pixel in line.iter() {
                                print!("{}", tones.chars().nth(*pixel as usize / 25).unwrap());
                            }
                            println!();
                        }
                    }
                } else {
                    warn!("Image from an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::InitClient {
                client_id,
                usernames,
            } => {
                users.self_id = client_id;
                users.names = usernames;
            }
        }
    }
}

fn handle_terminal_messages(
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut app_exit_events: EventWriter<AppExit>,
    client: Res<Client>,
) {
    while let Ok(message) = terminal_messages.try_recv() {
        if message == "quit" {
            app_exit_events.send(AppExit);
        } else if message == "image" {
            client.connection().try_send_message(ClientMessage::Image {
                image: vec![
                    vec![00, 00, 00, 00, 00, 00, 00, 00, 00, 00],
                    vec![00, 10, 10, 10, 10, 10, 10, 10, 10, 00],
                    vec![00, 10, 20, 20, 20, 20, 20, 20, 10, 00],
                    vec![00, 10, 20, 30, 30, 30, 30, 20, 10, 00],
                    vec![00, 10, 20, 30, 40, 40, 30, 20, 10, 00],
                    vec![00, 10, 20, 30, 40, 40, 30, 20, 10, 00],
                    vec![00, 10, 20, 30, 30, 30, 30, 20, 10, 00],
                    vec![00, 10, 20, 20, 20, 20, 20, 20, 10, 00],
                    vec![00, 10, 10, 10, 10, 10, 10, 10, 10, 00],
                    vec![00, 00, 00, 00, 00, 00, 00, 00, 00, 00],
                ],
            });
        } else {
            client
                .connection()
                .try_send_message(ClientMessage::ChatMessage { message });
        }
    }
}

fn start_terminal_listener(mut commands: Commands) {
    let (from_terminal_sender, from_terminal_receiver) = mpsc::channel::<String>(100);

    thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        from_terminal_sender
            .try_send(buffer.trim_end().to_string())
            .unwrap();
    });

    commands.insert_resource(TerminalReceiver(from_terminal_receiver));
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
