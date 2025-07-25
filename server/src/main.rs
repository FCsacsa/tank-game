// Allow dead code while we are still actively developing
#![allow(dead_code, clippy::type_complexity)]

use std::{fs::read_to_string, net::UdpSocket, path::Path};

use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    input::{common_conditions::input_toggle_active, keyboard::KeyCode},
};

/// Holds the server configuration struct.
mod config;
/// Helpful debugging methods to display various info during runtime.
mod debug;
/// Holds the [`Component`](bevy::ecs::component::Component)s for the server.
mod entities;
/// The JSON representation of maps.
mod map;
/// Game systems.
mod systems;
/// Show leaderboard.
mod ui;
/// Collection of useful functions.
mod util;

use config::Config;
use debug::{do_bounds, do_debug, do_normals, do_spawns, draw_bounds, draw_normals, draw_spawns};
use entities::Socket;
use map::Maps;
use systems::{
    apply_controls, bullet_bullet_collision, bullet_wall_collision, listen_socket, load_map,
    move_bullets, move_tanks, move_turrets, player_disconnect, player_respawn, send_state,
    setup_camera, shoot_countdown, tank_bullet_collision, tank_tank_collision,
};
use ui::show_leaderboard;

use crate::ui::setup_leaderboard;

fn main() {
    // load config
    let config: Config = serde_json::from_str(
        &read_to_string("./assets/config.jsonc")
            .unwrap_or_default()
            .lines()
            .map(|l| if l.trim().starts_with("//") { "" } else { l })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .inspect(|_| println!("correct format"))
    .inspect_err(|err| println!("Incorrect config:\n{err}"))
    .unwrap_or_default();
    // bind socket
    let socket = UdpSocket::bind(("127.0.0.1", 4000)).unwrap();
    socket.set_nonblocking(true).unwrap();

    let basedir = Path::new(&config.map_dir);
    let maps = config
        .map_paths
        .iter()
        .flat_map(|path| {
            serde_json::from_str(
                &read_to_string(basedir.join(path))
                    .inspect_err(|err| println!("Map {path} not found:\n{err}"))
                    .unwrap_or_default()
                    .lines()
                    .map(|l| if l.trim().starts_with("//") { "" } else { l })
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .inspect_err(|err| println!("Reading map '{path}' failed with:\n{err}"))
        })
        .collect::<Vec<_>>();
    assert!(!maps.is_empty(), "At least one map has to be loaded.");

    App::new()
        .insert_resource(Socket(socket))
        .insert_resource(config)
        .insert_resource(Maps {
            loaded: maps,
            current: None,
        })
        .add_systems(Startup, (setup_camera, load_map, setup_leaderboard))
        .add_systems(FixedUpdate, (listen_socket, show_leaderboard))
        .add_systems(
            Update,
            (
                (
                    apply_controls,
                    (move_tanks, move_turrets, move_bullets),
                    (
                        tank_bullet_collision,
                        tank_tank_collision,
                        bullet_bullet_collision,
                        bullet_wall_collision,
                    )
                        .chain(),
                    (player_respawn, shoot_countdown, player_disconnect),
                )
                    .run_if(input_toggle_active(false, KeyCode::Space)),
                send_state,
            )
                .chain(),
        )
        .add_systems(Update, draw_normals.run_if(do_debug.or(do_normals)))
        .add_systems(Update, draw_bounds.run_if(do_debug.or(do_bounds)))
        .add_systems(Update, draw_spawns.run_if(do_debug.or(do_spawns)))
        .add_plugins(DefaultPlugins)
        .run();
}
