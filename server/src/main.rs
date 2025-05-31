// Allow dead code while we are still actively developing
#![allow(dead_code)]

use std::net::UdpSocket;
use std::path::Path;

use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::AssetServer,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{component::Component, schedule::IntoSystemConfigs, system::Query},
    math::Vec2,
    prelude::{Camera2d, Commands, Msaa, OrthographicProjection, Res},
    time::{Fixed, Time},
    utils::default,
    DefaultPlugins,
};
use iyes_perf_ui::{
    prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot},
    PerfUiPlugin,
};

mod map;
mod util;
use map::{wall_collision, Map};
mod tank;
use messages::ClientMessages;
use tank::{move_tanks, update_turrets, Tank};

#[derive(Component)]
struct Socket {
    udp: UdpSocket,
    ports: Vec<u16>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((move_tanks, wall_collision).chain(), update_turrets),
        )
        .add_systems(FixedUpdate, listen_socket)
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // set up server socket
    let server_port = std::env::var("SELF_PORT")
        .unwrap_or("4000".to_owned())
        .parse()
        .unwrap_or(4000);
    let socket =
        UdpSocket::bind(("127.0.0.1", server_port)).expect("[ERR] - could not bind server port");
    socket
        .set_nonblocking(true)
        .expect("[ERR] - could not set non-blocking mode");
    commands.spawn(Socket {
        udp: socket,
        ports: vec![],
    });

    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scale: 0.8,
            ..OrthographicProjection::default_2d()
        },
        Msaa::Off,
    ));

    Map::load_map_from_path(Path::new("assets/map.jsonc"))
        .unwrap()
        .setup(&mut commands, &asset_server);

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    Tank::setup(
        "tank_body.png",
        "tank_turret.png",
        Vec2::new(-113.0, 0.0),
        Vec2::new(100.0, 100.0),
        &mut commands,
        &asset_server,
    );

    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            values_col_width: 32.0,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}

fn listen_socket(mut sockets: Query<&mut Socket>) {
    for mut socket in &mut sockets {
        let mut buf = [0; 16];
        if let Ok(n_bytes) = socket.udp.recv(&mut buf) {
            let msg = ClientMessages::from(buf);
            println!("[LOG] - got input ({msg}) with length {n_bytes}");
            match msg {
                ClientMessages::ConnectMessage { port } => socket.ports.push(port),
                ClientMessages::ControlMessage {
                    target_acceleration,
                    turret_acceleration,
                    shoot,
                } => todo!(),
            }
        }
    }
}
