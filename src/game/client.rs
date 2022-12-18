use crate::GameState;
use bevy::{ecs::schedule::ShouldRun, prelude::*, utils::HashMap};
use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig, RenetError},
    RenetClientPlugin,
};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin::default())
            .add_system(panic_on_error_system)
            .add_system(recevice_messages.with_run_criteria(has_resource::<RenetClient>))
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(disconnect_client));
    }
}

fn disconnect_client(mut client: ResMut<RenetClient>) {
    client.disconnect();
}

fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn has_resource<T: Resource>(resource: Option<Res<T>>) -> ShouldRun {
    match resource.is_some() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

pub struct Client {
    pub client: RenetClient,
    pub usernames: HashMap<u64, String>,
}

impl Client {
    pub fn new(ip: String, _: String) -> RenetClient {
        let client_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server_addr = ip.parse().unwrap();
        let socket = UdpSocket::bind(client_addr).unwrap();
        let connection_config = RenetConnectionConfig::default();

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let authentication = ClientAuthentication::Unsecure {
            protocol_id: 0,
            client_id: 10,
            server_addr,
            user_data: None,
        };

        info!("Client connected to {}", server_addr);

        RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
    }
}

fn recevice_messages(mut client: ResMut<RenetClient>) {
    // client.send_message(DefaultChannel::Reliable, "hello");

    if let Some(messages) = client.receive_message(DefaultChannel::Reliable) {
        println!("Received {:?} messages", messages);
    }
}
