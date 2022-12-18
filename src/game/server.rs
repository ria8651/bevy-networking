use bevy::{ecs::schedule::ShouldRun, prelude::*, utils::HashMap};
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
    },
    RenetServerPlugin,
};
use std::{net::UdpSocket, time::SystemTime};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default())
            .add_system(send_messages.with_run_criteria(has_resource::<RenetServer>));
    }
}

fn has_resource<T: Resource>(resource: Option<Res<T>>) -> ShouldRun {
    match resource.is_some() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

pub struct Server {
    pub server: RenetServer,
    pub usernames: HashMap<u64, String>,
}

impl Server {
    pub fn new(_: String) -> RenetServer {
        let socket = UdpSocket::bind("127.0.0.1:1234").unwrap();
        let server_addr = socket.local_addr().unwrap();
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

        RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
    }
}

fn send_messages(mut server: ResMut<RenetServer>) {
    // client.send_message(DefaultChannel::Reliable, "hello");

    server.send_message(10, DefaultChannel::Reliable, "Hello");
}
