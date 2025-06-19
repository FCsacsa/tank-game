// Allow dead code while we are still actively developing
#![allow(dead_code)]

use std::{net::UdpSocket, path::Path, time::Duration};

use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::AssetServer,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
        system::{Query, QueryLens, ResMut, Resource},
    },
    hierarchy::Children,
    math::Vec2,
    prelude::{Camera2d, Commands, Msaa, OrthographicProjection, Res},
    time::{Fixed, Time},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};
use bullet::Bullet;
use iyes_perf_ui::{
    prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot},
    PerfUiPlugin,
};
use messages::{
    client::ClientMessages,
    server::{self, ServerMessages},
};

mod bullet;
mod map;
mod tank;
mod util;

use map::{wall_collision, Map};
use tank::{move_tanks, update_turrets, Tank, TankData, TurretData};
use util::forget_z_arr;

#[derive(Component, Resource)]
struct Socket {
    udp: UdpSocket,
    ports: Vec<(u16, u128)>,
}

fn main() {
    let server_port = std::env::var("SELF_PORT")
        .unwrap_or("4000".to_owned())
        .parse()
        .unwrap_or(4000);
    let socket =
        UdpSocket::bind(("127.0.0.1", server_port)).expect("[ERR] - could not bind server port");
    socket
        .set_nonblocking(true)
        .expect("[ERR] - could not set non-blocking mode");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .insert_resource(Socket {
            udp: socket,
            ports: vec![],
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (Bullet::shoot, Bullet::move_bullets, Bullet::hit).chain(),
                ((move_tanks, wall_collision).chain(), update_turrets),
                send_state,
            )
                .chain(),
        )
        .add_systems(FixedUpdate, listen_socket)
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

fn send_state(
    socket: Res<Socket>,
    mut transforms: Query<&Transform>,
    mut tanks_data: Query<(&TankData, &Children)>,
) {
    let mut tanks = vec![];

    let mut tanks_data: QueryLens<(&TankData, &Children, &Transform)> =
        tanks_data.join_filtered(&mut transforms);
    for (_, children, transform) in &tanks_data.query() {
        let turret = transforms
            .get(*children.first().expect("tank has a turret as child"))
            .expect("turret has a transform")
            .up();
        let turret_in_world = turret;
        tanks.push(server::Tank {
            position: forget_z_arr(transform.translation),
            tank_direction: forget_z_arr(transform.up().into()),
            turret_direction: forget_z_arr(turret_in_world.into()),
        });
    }

    for (port, secret) in &socket.ports {
        let msg = Vec::from(&ServerMessages::State {
            secret: *secret,
            tanks: tanks.clone(),
            bullets: vec![],
        });
        if let Err(err) = socket.udp.send_to(&msg, ("127.0.0.1", *port)) {
            println!("[ERR] could not send state to {port}: {err}");
        }
    }
}

fn listen_socket(
    mut socket: ResMut<Socket>,
    mut tanks: Query<&mut TankData>,
    mut turrets: Query<&mut TurretData>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut buf = [0; 32];
    while let Ok(n_bytes) = socket.udp.recv(&mut buf) {
        if let Ok(msg) = ClientMessages::try_from(&buf[..]) {
            println!("[LOG] - got input ({msg:?}) with length {n_bytes}");
            match msg {
                ClientMessages::Connect { self_port } => {
                    let mut secret: u128 = rand::random();
                    while socket.ports.iter().any(|(_, s)| *s == secret) {
                        secret = rand::random();
                    }
                    socket.ports.push((self_port, rand::random()));
                    Tank::setup(
                        self_port,
                        "tank_body.png",
                        "tank_turret.png",
                        Vec2::new(0.0, 0.0),
                        Vec2::new(100.0, 100.0),
                        &mut commands,
                        &asset_server,
                    );
                }
                ClientMessages::Control {
                    self_port,
                    secret,
                    tracks_acceleration_target,
                    turret_acceleration_target,
                    shoot,
                } => {
                    if socket
                        .ports
                        .iter()
                        .any(|(p, s)| *p == self_port && *s == secret)
                    {
                        if let Some(mut data) = tanks.iter_mut().find(|d| d.player == self_port) {
                            data.set_acceleration(tracks_acceleration_target.into());
                            data.connection_timeout = Duration::from_secs(0);
                            data.shoot = shoot;
                        }
                        if let Some(mut data) = turrets.iter_mut().find(|d| d.player == self_port) {
                            data.set_acceleration(turret_acceleration_target);
                        }
                    }
                }
            }
        } else {
            println!("Ill-formatted message: {buf:?}");
        }
    }
}
