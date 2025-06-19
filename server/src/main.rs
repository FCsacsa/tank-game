use std::{fs::read_to_string, net::UdpSocket};

use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup, Update},
    ecs::schedule::IntoScheduleConfigs,
};

mod config;
mod entities;
mod map;
mod systems;
mod util;

use config::Config;
use entities::Socket;
use map::{Map, Maps};
use systems::{
    bullet_bullet_collision, bullet_wall_collision, listen_socket, load_map, move_bullets, move_tanks, player_disconnect, player_respawn, send_state, setup_camera, tank_bullet_collision, tank_tank_collision, tank_wall_collision
};

fn main() {
    // load config
    let config = Config::default();
    // bind socket
    let socket = UdpSocket::bind(("127.0.0.1", 4000)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut maps: Vec<Map> = vec![];
    for map_path in &config.map_paths {
        maps.push(
            serde_json::from_str(&read_to_string(map_path).expect("Missing map '{map_path}'"))
                .expect("Incorrect map format '{map_path}'"),
        );
    }

    App::new()
        .insert_resource(Socket(socket))
        .insert_resource(config)
        .insert_resource(Maps { loaded: maps, current: None })
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, load_map)
        .add_systems(FixedUpdate, listen_socket)
        .add_systems(
            Update,
            (
                (move_tanks, move_bullets),
                (
                    tank_bullet_collision,
                    tank_tank_collision,
                    tank_wall_collision,
                    bullet_bullet_collision,
                    bullet_wall_collision,
                )
                    .chain(),
                (player_respawn, player_disconnect),
                send_state,
            )
                .chain(),
        )
        .add_plugins(DefaultPlugins)
        .run();
}
