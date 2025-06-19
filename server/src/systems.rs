use std::{f32::consts::PI, net::IpAddr, str::FromStr, time::Duration};

use bevy::{
    asset::AssetServer,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        entity::Entity,
        hierarchy::{ChildOf, Children},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    math::Quat,
    time::Time,
    transform::components::Transform,
};
use messages::{
    client::ClientMessages,
    server::{self, ServerMessages},
};

use crate::{
    config::Config,
    entities::{Bullet, Player, Socket, Spawn, Tank, Turret, Wall},
    map::Maps,
    util::forget_z_arr,
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    // fix scaling
}

pub fn load_map(commands: Commands, asset_server: Res<AssetServer>, mut maps: ResMut<Maps>) {
    let index = rand::random_range(0..maps.loaded.len());
    maps.loaded[index].spawn(commands, asset_server);
    maps.current = Some(index);
}

pub fn listen_socket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    socket: Res<Socket>,
    maps: Res<Maps>,
    spawns: Query<&Transform, With<Spawn>>,
    mut players: Query<(&mut Player, &Children)>,
    mut tanks: Query<(&mut Tank, &Children)>,
    mut turrets: Query<&mut Turret>,
) {
    let mut buf = [0; 32];
    while let Ok((_, addr)) = socket.0.recv_from(&mut buf) {
        if addr.ip() != IpAddr::from_str("127.0.0.1").unwrap() {
            println!("[ERR] - message from the wrong address: {addr}");
            continue;
        }
        if let Ok(msg) = ClientMessages::try_from(&buf[..]) {
            match msg {
                ClientMessages::Connect { self_port } => {
                    if addr.port() == self_port {
                        Player::spawn(
                            self_port,
                            rand::random(),
                            spawns
                                .iter()
                                .skip(rand::random_range(0..spawns.iter().count().max(1)))
                                .next()
                                .map(|t| t.translation)
                                .unwrap_or([0.0, 0.0, 0.0].into()),
                            "tank_body.png".to_owned(),
                            "tank_turret.png".to_owned(),
                            "tank_bullet.png".to_owned(),
                            &mut commands,
                            &asset_server,
                        )
                    }
                }
                ClientMessages::Control {
                    self_port,
                    secret,
                    tracks_acceleration_target,
                    turret_acceleration_target,
                    shoot,
                } => {
                    if addr.port() == self_port {
                        if let Some((mut player, children)) = players
                            .iter_mut()
                            .find(|(p, _)| p.port == self_port && p.secret == secret)
                        {
                            let (mut tank, children) = tanks.get_mut(children[0]).unwrap();
                            let mut turret = turrets.get_mut(children[0]).unwrap();
                            tank.track_accelerations = tracks_acceleration_target.into();
                            turret.acceleration = turret_acceleration_target;
                            player.timeout = Duration::from_micros(0);
                        }
                    }
                }
            }
        }
    }
}

pub fn send_state(
    socket: Res<Socket>,
    players: Query<&Player>,
    tanks: Query<(&Transform, &Children), With<Tank>>,
    turrets: Query<&Transform, With<Turret>>,
    bullets: Query<(&Bullet, &Transform)>,
) {
    let tanks: Vec<_> = tanks
        .iter()
        .map(|(tank, turret)| server::Tank {
            position: forget_z_arr(tank.translation),
            tank_direction: forget_z_arr(tank.up().as_vec3()),
            turret_direction: forget_z_arr(
                tank.rotation * turrets.get(turret[0]).unwrap().up().as_vec3(),
            ),
        })
        .collect();
    let bullets: Vec<_> = bullets
        .iter()
        .map(|(bullet, transform)| server::Bullet {
            position: forget_z_arr(transform.translation),
            direction: bullet.direction.to_array(),
        })
        .collect();

    for player in players {
        let msg = ServerMessages::State {
            secret: player.secret,
            tanks: tanks.clone(),
            bullets: bullets.clone(),
        };

        // We do not care too much about errors.
        let _ = socket.0.send_to(&msg.to_vec(), ("127.0.0.1", player.port));
    }
}

pub fn move_tanks(time: Res<Time>, config: Res<Config>, tanks: Query<(&mut Tank, &mut Transform)>) {
    for (mut tank, mut transform) in tanks {
        // update speed
        let new_speed = (tank.track_accelerations * time.delta_secs() + tank.track_velocities)
            .clamp(-config.max_track_speed, config.max_track_speed);
        tank.track_velocities = new_speed;

        if (tank.track_velocities.x - tank.track_velocities.y).abs() < f32::EPSILON {
            // only forward
            let move_direction = transform.up();
            transform.translation += move_direction
                * (tank.track_velocities.x + tank.track_velocities.y)
                * 0.5
                * time.delta_secs();
        } else {
            // do turn
            let radius = (config.tank_width * tank.track_velocities.y)
                / (tank.track_velocities.x - tank.track_velocities.y);
            let axis =
                transform.right() * (config.tank_width * 0.5 + radius) + transform.translation;
            let mut angle = tank.track_velocities.y / (2.0 * PI * radius) * time.delta_secs();
            if !angle.is_normal() {
                angle = tank.track_velocities.x / (2.0 * PI * (config.tank_width + radius))
                    * time.delta_secs();
            }
            transform.rotate_around(axis, Quat::from_rotation_z(-angle));
        }
    }
}

pub fn move_bullets(time: Res<Time>, bullets: Query<(&Bullet, &mut Transform)>) {}

pub fn tank_tank_collision(
    mut tanks: Query<(&mut Transform, &ChildOf), With<Tank>>, // potentially these could influence the tanks speed?
    mut player: Query<&mut Player>,
) {
    // Blow up both, add death to player.
}

pub fn tank_wall_collision(mut walls: Query<&Wall>, mut tanks: Query<&mut Tank>) {}

pub fn tank_bullet_collision(
    mut tanks: Query<(&Tank, &ChildOf, &Transform), Without<Bullet>>,
    mut player: Query<&mut Player>,
    mut bullets: Query<(&Bullet, &Transform), Without<Tank>>,
    mut commands: Commands,
) {
    for (tank, parent, transform) in &tanks {
        for (bullet, bullet_pos) in &bullets {
            let distance = transform.translation - bullet_pos.translation;
        }
    }
}

pub fn bullet_wall_collision() {}

pub fn bullet_bullet_collision() {}

pub fn player_respawn() {}

pub fn player_disconnect(
    time: Res<Time>,
    config: Res<Config>,
    players: Query<(&mut Player, Entity)>,
    mut commands: Commands,
) {
    for (mut player, entity) in players {
        player.timeout += time.delta();

        if player.timeout > config.inactivity_timeout {
            // TODO: Save score.
            // TODO: Send message to player.
            // Despawn player.
            commands.entity(entity).despawn();
        }
    }
}
