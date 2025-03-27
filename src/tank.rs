use std::f32::{consts::PI, EPSILON};

use bevy::asset::AssetServer;
use bevy::math::{Quat, Vec2};
use bevy::prelude::{BuildChildren, Bundle, Commands, Component, Query, Res, Transform};
use bevy::{sprite::Sprite, time::Time};

const WIDTH: f32 = 20.0;
const HEIGTH: f32 = 25.0;

const TANK_MAX_SPEED: f32 = 500.0;
const TANK_MAX_ACCELERATION: f32 = 100.0;

const TURRET_MAX_SPEED: f32 = 2.0;
const TURRET_MAX_ACCELERATION: f32 = 0.5;

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
            transform: Transform::from_xyz(0.0, 3.0, EPSILON),
        }
    }
}

#[derive(Component)]
pub struct TankData {
    speed: Vec2,
    acceleration: Vec2,
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
        speed: Vec2,
        acceleration: Vec2,
        body_sprite: Sprite,
        start_position: Transform,
    ) -> Self {
        Self {
            data: TankData {
                speed,
                acceleration,
            },
            sprite: body_sprite,
            transform: start_position,
        }
    }

    pub fn setup(
        body_path: &str,
        turret_path: &str,
        start_position: Vec2,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
    ) {
        commands
            .spawn(Tank::new(
                Vec2::new(0.0, 0.0),
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

pub fn move_tanks(time: Res<Time>, mut tanks: Query<(&mut TankData, &mut Transform)>) {
    for (mut data, mut transform) in &mut tanks {
        // update speed
        let new_speed = (data.acceleration * time.delta_secs() + data.speed).clamp(
            Vec2::new(-TANK_MAX_SPEED, -TANK_MAX_SPEED),
            Vec2::new(TANK_MAX_SPEED, TANK_MAX_SPEED),
        );
        data.speed = new_speed;

        if (data.speed.x - data.speed.y).abs() < EPSILON {
            // only forward
            let move_direction = transform.up();
            transform.translation +=
                move_direction * (data.speed.x + data.speed.y) * 0.5 * time.delta_secs();
            return;
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
}
