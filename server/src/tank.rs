use std::f32::consts::PI;
use std::ops::Index;
use std::time::Duration;

use bevy::asset::AssetServer;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::{Children, DespawnRecursiveExt};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{BuildChildren, Bundle, Commands, Component, Query, Res, Transform};
use bevy::{sprite::Sprite, time::Time};

use crate::Socket;

const WIDTH: f32 = 20.0;
const _HEIGTH: f32 = 25.0;
pub const DIAMETER: f32 = 25.0;
pub const DIAMETER_SQUARED: f32 = 625.0;

pub const TANK_MAX_SPEED: f32 = 500.0;
const TANK_MAX_ACCELERATION: f32 = 100.0;

const TURRET_MAX_SPEED: f32 = 2.0;
const TURRET_MAX_ACCELERATION: f32 = 0.5;

const INACTIVE_TIMEOUT: Duration = Duration::from_secs(5);

const DISCONNECTED: [u8; 1] = [2];

#[derive(Component, Debug)]
pub struct TurretData {
    velocity: f32,
    acceleration: f32,
}

#[derive(Bundle)]
pub struct Turret {
    data: TurretData,
    sprite: Sprite,
    transform: Transform,
}

impl Turret {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            data: TurretData {
                velocity: 0.0,
                acceleration: 0.0,
            },
            sprite,
            transform: Transform::from_xyz(0.0, 3.0, f32::EPSILON),
        }
    }
}

#[derive(Component)]
pub struct TankData {
    player: u16,
    speed: Vec2,
    acceleration: Vec2,
    timeout: Duration,
}

impl TankData {
    pub fn set_acceleration(mut self, new_value: Vec2) {
        self.acceleration = new_value.clamp(
            Vec2::new(-TANK_MAX_ACCELERATION, -TANK_MAX_ACCELERATION),
            Vec2::new(TANK_MAX_ACCELERATION, TANK_MAX_ACCELERATION),
        );
    }
}

#[derive(Bundle)]
pub struct Tank {
    data: TankData,
    sprite: Sprite,
    transform: Transform,
}

impl Tank {
    pub fn new(
        player: u16,
        speed: Vec2,
        acceleration: Vec2,
        body_sprite: Sprite,
        start_position: Transform,
    ) -> Self {
        Self {
            data: TankData {
                player,
                speed,
                acceleration,
                timeout: Duration::from_micros(0),
            },
            sprite: body_sprite,
            transform: start_position,
        }
    }

    pub fn setup(
        player: u16,
        body_path: &str,
        turret_path: &str,
        start_position: Vec2,
        start_velocity: Vec2,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
    ) {
        commands
            .spawn(Tank::new(
                player,
                start_velocity,
                Vec2::new(0.0, 0.0),
                Sprite::from_image(asset_server.load(body_path)),
                Transform::from_xyz(start_position.x, start_position.y, 0.0),
            ))
            .with_child(Turret::new(Sprite::from_image(
                asset_server.load(turret_path),
            )));
    }
}

pub fn update_turrets(time: Res<Time>, mut turrets: Query<(&mut TurretData, &mut Transform)>) {
    for (mut data, mut transform) in &mut turrets {
        // update speed
        data.velocity = (data.acceleration * time.delta_secs() + data.velocity)
            .clamp(-TURRET_MAX_SPEED, TURRET_MAX_SPEED);

        // rotate
        let axis = transform.translation - 6.5 * transform.up();
        transform.rotate_around(
            axis,
            Quat::from_rotation_z(data.velocity * time.delta_secs()),
        );

        // TODO: add friction
    }
}

pub fn move_tanks(
    time: Res<Time>,
    mut tanks: Query<(Entity, &mut TankData, &mut Transform)>,
    mut socket: Query<&mut Socket>,
    mut commands: Commands,
) {
    for (entity_id, mut data, mut transform) in &mut tanks {
        // update timeout
        data.timeout += time.delta();
        if data.timeout > INACTIVE_TIMEOUT {
            // do not send to them anymare
            socket.iter().for_each(|s| {
                if let Err(err) = s.udp.send_to(&DISCONNECTED, ("127.0.0.1", data.player)) {
                    println!(
                        "[ERR] - something failed while disconnecting {}: {err}",
                        data.player
                    )
                }
            });
            socket
                .iter_mut()
                .for_each(|mut s| s.ports.retain(|p| *p != data.player));
            // despawn
            commands.entity(entity_id).despawn_recursive();
        }

        // update speed
        let new_speed = (data.acceleration * time.delta_secs() + data.speed).clamp(
            Vec2::new(-TANK_MAX_SPEED, -TANK_MAX_SPEED),
            Vec2::new(TANK_MAX_SPEED, TANK_MAX_SPEED),
        );
        data.speed = new_speed;

        if (data.speed.x - data.speed.y).abs() < f32::EPSILON {
            // only forward
            let move_direction = transform.up();
            transform.translation +=
                move_direction * (data.speed.x + data.speed.y) * 0.5 * time.delta_secs();
        } else {
            // do turn
            let radius = (WIDTH * data.speed.y) / (data.speed.x - data.speed.y);
            let axis = transform.right() * (WIDTH * 0.5 + radius) + transform.translation;
            let mut angle = data.speed.y / (2.0 * PI * radius) * time.delta_secs();
            if !angle.is_normal() {
                angle = data.speed.x / (2.0 * PI * (WIDTH + radius)) * time.delta_secs();
            }
            transform.rotate_around(axis, Quat::from_rotation_z(-angle));
        }

        // TODO: add friction?
    }

    // NOTE: for simplicity, tanks have a circle as their bounding box
    let tank_count = tanks.iter().count();
    let mut corrections = tanks
        .iter()
        .map(|_| Vec3::new(0.0, 0.0, 0.0))
        .collect::<Vec<_>>();
    for (i, (_, _, pos_1)) in tanks.iter().enumerate() {
        for (j, (_, _, pos_2)) in tanks.iter().enumerate().skip(i + 1) {
            let distance = pos_1.translation - pos_2.translation;
            if distance.length_squared() < DIAMETER_SQUARED {
                let dir = distance.normalize();
                corrections[i] -= 0.5 * (distance - (dir * DIAMETER));
                corrections[j] += 0.5 * (distance - (dir * DIAMETER));
            }
        }
    }

    let mut correction_iter = corrections.iter();
    for (_, _, mut transform) in &mut tanks {
        let correction = correction_iter
            .next()
            .expect("Should have the same size by definition.");
        transform.translation += correction;
    }
}
