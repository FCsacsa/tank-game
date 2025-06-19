use std::{net::UdpSocket, time::Duration};

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Res},
    },
    math::{Dir2, Vec2, Vec3},
    render::view::Visibility,
    sprite::Sprite,
    transform::components::Transform,
};
use serde::Serialize;

/// For easy access in the systems, we bundle the [`UdpSocket`] as a [`Resource`].
#[derive(Component, Resource)]
pub struct Socket(pub UdpSocket);

/// Struct corresponding to a connected player.
/// It shall contain any stats of the player, to allow for upgrades.
#[derive(Component)]
pub struct Player {
    /// The port where we can send the messages to the player.
    pub port: u16,
    /// Some rudamentary form of security, as random generated secret upon connection.
    pub secret: u128,
    /// Timer for inactivity.
    pub timeout: Duration,

    /// Counter for the number of deaths.
    pub deaths: u32,
    /// Respawn delay
    pub respawn_timer: Option<Duration>,

    tank_sprite_path: String,
    turret_sprite_path: String,
    bullet_sprite_path: String,
}

impl Player {
    pub fn spawn(
        port: u16,
        secret: u128,
        position: Vec3,
        tank_sprite_path: String,
        turret_sprite_path: String,
        bullet_sprite_path: String,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
    ) {
        commands
            .spawn((
                Player {
                    port,
                    secret,
                    timeout: Duration::from_micros(0),
                    tank_sprite_path: tank_sprite_path.clone(),
                    turret_sprite_path: turret_sprite_path.clone(),
                    bullet_sprite_path,
                    deaths: 0,
                    respawn_timer: None,
                },
                Transform::default(),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Tank {
                            track_velocities: [0.0, 0.0].into(),
                            track_accelerations: [0.0, 0.0].into(),
                        },
                        Transform::from_translation(position),
                        Sprite::from_image(asset_server.load(tank_sprite_path)),
                    ))
                    .with_child((
                        Turret {
                            velocity: 0.0,
                            acceleration: 0.0,
                        },
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        Sprite::from_image(asset_server.load(turret_sprite_path)),
                    ));
            });
    }
}

/// Holds physics data for a tank.
/// Should only be spawned as a [`ChildOf`](bevy::prelude::ChildOf) a [`Player`].
#[derive(Component)]
pub struct Tank {
    /// Velocities of the two tank treads.
    pub track_velocities: Vec2,
    /// Accelerations of the two tank treads.
    pub track_accelerations: Vec2,
}

#[derive(Component)]
pub struct Turret {
    /// Rotational speed of the turret (in radians per second).
    pub velocity: f32,
    /// Acceleration of the turret.
    pub acceleration: f32,
}

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec2,
}

#[derive(Component, Resource)]
#[require(Transform)]
pub struct Map;

/// Data for a wall, should be bundled with a [`Transform`](bevy::transform::components::Transform) and a [`Sprite`](bevy::sprite::Sprite).
#[derive(Component)]
pub struct Wall {
    pub(crate) normal: Dir2,
    pub(crate) direction: Dir2,
    pub(crate) half_length: f32,
}

/// Marker for the spawn points in a [`Map`].
/// Should be bundled together with a [`Transform`](bevy::transform::components::Transform).
#[derive(Component)]
pub struct Spawn();
