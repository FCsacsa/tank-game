// Allow dead code while we are still actively developing
#![allow(dead_code, clippy::type_complexity)]

use std::{fs::read_to_string, net::UdpSocket, path::Path};

use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
};

mod config;
mod debug;
mod entities;
mod map;
mod systems;
mod util;

use config::Config;
use debug::{do_bounds, do_debug, do_normals, draw_bounds, draw_normals};
use entities::Socket;
use map::{Map, Maps};
use systems::{
    apply_controls, bullet_bullet_collision, bullet_wall_collision, listen_socket, load_map,
    move_bullets, move_tanks, player_disconnect, player_respawn, send_state, setup_camera,
    shoot_countdown, tank_bullet_collision, tank_tank_collision, tank_wall_collision,
};

fn main() {
    // load config
    let config = Config::default();
    // bind socket
    let socket = UdpSocket::bind(("127.0.0.1", 4000)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut maps: Vec<Map> = vec![];
    let basedir = Path::new(&config.map_dir);
    for map_path in &config.map_paths {
        let path = basedir.join(map_path);
        maps.push(
            serde_json::from_str(
                &read_to_string(path).unwrap_or_else(|_| panic!("Missing map '{map_path}'")),
            )
            .unwrap_or_else(|_| panic!("Incorrect map format '{map_path}'")),
        );
    }

    App::new()
        .insert_resource(Socket(socket))
        .insert_resource(config)
        .insert_resource(Maps {
            loaded: maps,
            current: None,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, load_map)
        .add_systems(FixedUpdate, listen_socket)
        .add_systems(
            Update,
            (
                apply_controls,
                (move_tanks, move_bullets),
                (
                    tank_bullet_collision,
                    tank_tank_collision,
                    tank_wall_collision,
                    bullet_bullet_collision,
                    bullet_wall_collision,
                )
                    .chain(),
                (player_respawn, shoot_countdown, player_disconnect),
                send_state,
            )
                .chain(),
        )
        .add_systems(Update, draw_normals.run_if(do_debug.or(do_normals)))
        .add_systems(Update, draw_bounds.run_if(do_debug.or(do_bounds)))
        .add_plugins(DefaultPlugins)
        .run();
}
