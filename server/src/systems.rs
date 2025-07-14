use bevy::{
    asset::AssetServer,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        entity::Entity,
        hierarchy::{ChildOf, Children},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut, command},
    },
    log,
    math::{Quat, Vec2},
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};
use messages::{
    client::ClientMessages,
    server::{self, ServerMessages},
};
use std::{f32::consts::PI, net::IpAddr, str::FromStr, time::Duration};

use crate::{
    config::Config,
    entities::{Bullet, Player, Socket, Spawn, Tank, Turret, Wall},
    map::Maps,
    util::{forget_z, forget_z_arr, with_z},
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn load_map(commands: Commands, asset_server: Res<AssetServer>, mut maps: ResMut<Maps>) {
    let index = rand::random_range(0..maps.loaded.len());
    maps.loaded[index].spawn(commands, asset_server);
    maps.current = Some(index);
}

pub fn listen_socket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
    socket: Res<Socket>,
    spawns: Query<&Transform, With<Spawn>>,
    mut players: Query<(&mut Player, Entity)>,
) {
    let mut buf = [0; 32];
    while let Ok((_, addr)) = socket.0.recv_from(&mut buf) {
        if addr.ip() != IpAddr::from_str("127.0.0.1").unwrap() {
            log::warn!("Got a message from outside IP: {addr}");
            continue;
        }
        if let Ok(msg) = ClientMessages::try_from(&buf[..]) {
            match msg {
                ClientMessages::Connect { self_port } => {
                    if addr.port() == self_port {
                        match players.iter_mut().find(|(p, _)| p.port == self_port) {
                            Some((mut player, entity)) => {
                                let position = spawns
                                    .iter()
                                    .skip(rand::random_range(0..spawns.iter().count().max(1)))
                                    .next()
                                    .map(|t| t.translation)
                                    .unwrap_or_default();
                                commands
                                    .entity(entity)
                                    .despawn_related::<Children>()
                                    .with_children(|parent| {
                                        parent
                                            .spawn((
                                                Tank {
                                                    track_max_velocity: player.track_max_velocity,
                                                    radius: player.tank_radius,
                                                    ..Default::default()
                                                },
                                                Transform::from_translation(position),
                                            ))
                                            .with_child((
                                                Turret {
                                                    ..Default::default()
                                                },
                                                Transform::default(),
                                            ));
                                    });
                                player.reset_input()
                            }
                            None => Player::spawn(
                                self_port,
                                rand::random(),
                                spawns
                                    .iter()
                                    .skip(rand::random_range(0..spawns.iter().count().max(1)))
                                    .next()
                                    .map(|t| t.translation)
                                    .unwrap_or_default(),
                                "tank_body.png".to_owned(),
                                "tank_turret.png".to_owned(),
                                "bullet.png".to_owned(),
                                &mut commands,
                                &config,
                                &asset_server,
                            ),
                        }
                    } else {
                        log::warn!(
                            "Got a message from port {} trying to immitate {}",
                            addr.port(),
                            self_port
                        );
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
                        if let Some((mut player, _)) = players
                            .iter_mut()
                            .find(|(p, _)| p.port == self_port && p.secret == secret)
                        {
                            player.tracks_acceleration_target = tracks_acceleration_target.into();
                            player.turret_acceleration_target = turret_acceleration_target;
                            player.shoot = shoot;
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
            direction: bullet.velocity.to_array(),
        })
        .collect();

    let mut msg = ServerMessages::State {
        secret: 0,
        tanks,
        bullets,
    };

    for player in players {
        msg.change_secret(player.secret);
        let res = socket.0.send_to(&msg.to_vec(), ("127.0.0.1", player.port));
        match res {
            Ok(_) => {}
            Err(err) => log::warn!("Sending to player @ {} failed with {}", player.port, err),
        }
    }
}

pub fn apply_controls(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut players: Query<(&mut Player, &Children)>,
    mut tanks: Query<(&mut Tank, &Transform, &Children), Without<Turret>>,
    mut turrets: Query<(&mut Turret, &Transform), Without<Tank>>,
) {
    for (mut player, children) in &mut players {
        if let Ok((mut tank, tank_transform, children)) = tanks.get_mut(children[0]) {
            tank.track_accelerations = player.tracks_acceleration_target;

            match turrets.get_mut(children[0]) {
                Ok((mut turret, turret_transform)) => {
                    turret.acceleration = player.turret_acceleration_target;
                    if player.shoot && player.shoot_timer.is_none() {
                        let direction = tank_transform.rotation * turret_transform.up().as_vec3();
                        let translation = tank_transform.translation
                            + (tank.radius + player.bullet_radius) * direction;
                        let velocity = forget_z(direction) * player.bullet_speed;
                        commands.spawn((
                            Bullet {
                                velocity,
                                radius: player.bullet_radius,
                            },
                            Sprite::from_image(asset_server.load(&player.bullet_sprite_path)),
                            Transform::from_translation(translation),
                        ));
                        player.shoot = false;
                        player.shoot_timer = Some(Duration::default());
                    }
                }
                Err(_) => log::error_once!("Tank without a turret."),
            }
        }
    }
}

pub fn move_tanks(time: Res<Time>, tanks: Query<(&mut Tank, &mut Transform)>) {
    for (mut tank, mut transform) in tanks {
        // update speed
        let new_speed = (tank.track_accelerations * time.delta_secs() + tank.track_velocities)
            .clamp(-tank.track_max_velocity, tank.track_max_velocity);
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
            let radius = (2.0 * tank.radius * tank.track_velocities.y)
                / (tank.track_velocities.x - tank.track_velocities.y);
            let axis = transform.right() * (tank.radius + radius) + transform.translation;
            let mut angle = tank.track_velocities.y / (2.0 * PI * radius) * time.delta_secs();
            if !angle.is_normal() {
                angle = tank.track_velocities.x / (2.0 * PI * (2.0 * tank.radius + radius))
                    * time.delta_secs();
            }
            transform.rotate_around(axis, Quat::from_rotation_z(-angle));
        }
    }
}

pub fn move_bullets(time: Res<Time>, bullets: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in bullets {
        transform.translation += with_z(bullet.velocity * time.delta_secs(), 0.0);
    }
}

pub fn tank_tank_collision(
    mut commands: Commands,
    mut tanks: Query<(Entity, &Tank, &Transform, &ChildOf)>, // potentially these could influence the tanks speed?
    mut players: Query<&mut Player>,
) {
    let mut delete = vec![];
    for (i, (entity, tank, transform, parent)) in tanks.iter().enumerate() {
        let mut push_this = false;
        for (other, other_tank, other_transform, other_parent) in tanks.iter().skip(i + 1) {
            if (transform.translation - other_transform.translation).length()
                < tank.radius + other_tank.radius
            {
                delete.push((other, other_parent));
                push_this = true;
            }
        }
        if push_this && !delete.contains(&(entity, parent)) {
            delete.push((entity, parent));
        }
    }

    for (entity, parent) in delete {
        players.get_mut(parent.parent()).unwrap().deaths += 1;
        commands.entity(entity).despawn();
    }
}

pub fn tank_wall_collision(
    walls: Query<(&Wall, &Transform), Without<Tank>>,
    mut tanks: Query<(&Tank, &mut Transform), Without<Wall>>,
) {
    for (tank, mut transform) in &mut tanks {
        let mut correction = Vec2::default();

        for (wall, wall_origin) in &walls {
            let wall_origin = forget_z(wall_origin.translation);
            let tank_origin = forget_z(transform.translation);
            // from wikipedia
            let in_wall_dist = (wall_origin - tank_origin).dot(wall.direction.as_vec2());
            let dist_vec = (wall_origin - tank_origin) - in_wall_dist * wall.direction;
            let out_wall_dist = dist_vec.length();

            if in_wall_dist.abs() <= wall.half_length + tank.radius && out_wall_dist < tank.radius {
                // tank too close, push it
                correction += wall.normal * (tank.radius - out_wall_dist);
            }
        }

        transform.translation += with_z(correction, 0.0);
    }
}

pub fn tank_bullet_collision(
    mut commands: Commands,
    mut player: Query<&mut Player>,
    mut tanks: Query<(&Tank, &ChildOf, &Transform), Without<Bullet>>,
    mut bullets: Query<(&Bullet, &Transform), Without<Tank>>,
) {
    for (tank, parent, transform) in &tanks {
        for (bullet, bullet_pos) in &bullets {
            let distance = transform.translation - bullet_pos.translation;

            if distance.length() < tank.radius + bullet.radius {
                log::info!("bullet hit!");
            }
        }
    }
}

pub fn bullet_wall_collision() {}

pub fn bullet_bullet_collision() {}

pub fn player_respawn(
    mut commands: Commands,
    config: Res<Config>,
    time: Res<Time>,
    mut players: Query<(&mut Player, Entity)>,
    spawns: Query<&Transform, With<Spawn>>,
) {
    for (mut player, entity) in &mut players {
        if let Some(timer) = &mut player.respawn_timer {
            *timer += time.delta();

            if config.respawn_delay <= *timer {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn((
                            Tank {
                                track_velocities: Vec2::default(),
                                track_accelerations: Vec2::default(),
                                radius: player.tank_radius,
                                track_max_velocity: player.track_max_velocity,
                            },
                            Transform::from_translation(
                                spawns
                                    .iter()
                                    .skip(rand::random_range(0..spawns.iter().count().max(1)))
                                    .next()
                                    .map(|t| t.translation)
                                    .unwrap_or_default(),
                            ),
                        ))
                        .with_child((Turret::default(), Transform::default()));
                });
            }
        }
    }
}

pub fn shoot_countdown(time: Res<Time>, mut players: Query<&mut Player>) {
    for mut player in &mut players {
        if let Some(mut timer) = player.shoot_timer {
            timer += time.delta();
            if player.shoot_delay <= timer {
                player.shoot_timer = None;
            }
        }
    }
}

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
