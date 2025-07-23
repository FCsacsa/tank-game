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

use crate::config::Config;

/// For easy access in the systems, we bundle the [`UdpSocket`] as a [`Resource`].
#[derive(Component, Resource)]
pub struct Socket(pub UdpSocket);

/// Struct corresponding to a connected player.
/// It shall contain any stats of the player, to allow for upgrades.
#[derive(Component, Default)]
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

    // sprite information
    pub tank_sprite_path: String,
    pub turret_sprite_path: String,
    pub bullet_sprite_path: String,

    // tank properties
    /// Radius of the tank's collision circle.
    pub tank_radius: f32,
    /// Maximum allowed track speed.
    pub track_max_velocity: Vec2,
    pub track_max_acceleration: Vec2,
    pub turret_max_velocity: f32,
    pub turret_max_acceleration: f32,

    // player last input
    pub tracks_acceleration_target: Vec2,
    pub turret_acceleration_target: f32,

    /// Player's last shoot input.
    pub shoot: bool,
    /// Timer since last shoot.
    pub shoot_timer: Option<Duration>,
    pub shoot_delay: Duration,

    pub bullet_radius: f32,
    pub bullet_speed: f32,
    pub bullet_max_bounces: i8,
}

impl Player {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        port: u16,
        secret: u128,
        position: Vec3,
        tank_sprite_path: String,
        turret_sprite_path: String,
        bullet_sprite_path: String,
        commands: &mut Commands,
        config: &Res<Config>,
        asset_server: &Res<AssetServer>,
    ) {
        commands
            .spawn((
                Player {
                    port,
                    secret,
                    tank_sprite_path: tank_sprite_path.clone(),
                    turret_sprite_path: turret_sprite_path.clone(),
                    bullet_sprite_path,
                    tank_radius: config.tank_radius,
                    track_max_velocity: config.track_max_velocity,
                    track_max_acceleration: config.track_max_acceleration,
                    turret_max_velocity: config.turret_max_velocity,
                    turret_max_acceleration: config.turret_max_acceleration,
                    shoot_delay: config.shoot_delay,
                    bullet_radius: config.bullet_radius,
                    bullet_speed: config.bullet_speed,
                    bullet_max_bounces: config.bullet_max_bounces,
                    ..Default::default()
                },
                Transform::default(),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Tank {
                            radius: config.tank_radius,
                            track_max_velocity: config.track_max_velocity,
                            ..Default::default()
                        },
                        Transform::from_translation(position),
                        Sprite::from_image(asset_server.load(tank_sprite_path)),
                    ))
                    .with_child((
                        Turret {
                            max_velocity: config.turret_max_velocity,
                            max_acceleration: config.turret_max_acceleration,
                            ..Default::default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        Sprite::from_image(asset_server.load(turret_sprite_path)),
                    ));
            });
    }

    pub fn reset_input(&mut self) {
        self.shoot = false;
        self.tracks_acceleration_target = Default::default();
        self.turret_acceleration_target = 0.0;
    }

    pub fn death(&mut self) {
        self.reset_input();
        self.deaths += 1;
        self.respawn_timer = Some(Duration::new(0,0));
    }
}

/// Holds physics data for a tank.
/// Should only be spawned as a [`ChildOf`](bevy::prelude::ChildOf) a [`Player`].
#[derive(Component, Default)]
pub struct Tank {
    /// Velocities of the two tank treads.
    pub track_velocities: Vec2,
    /// Accelerations of the two tank treads.
    pub track_accelerations: Vec2,
    /// Size of the tank
    pub radius: f32,
    pub track_max_velocity: Vec2,
}

#[derive(Component, Default)]
pub struct Turret {
    /// Rotational speed of the turret (in radians per second).
    pub velocity: f32,
    /// Maximum velocity.
    pub max_velocity: f32,
    /// Acceleration of the turret.
    pub acceleration: f32,
    /// Maximum acceleration.
    pub max_acceleration: f32,
}

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub radius: f32,
    max_bounces: i8,
    bounces: i8,
}

impl Bullet {
    pub fn new(velocity: Vec2, radius: f32, max_bounces: i8) -> Self {
        Self {
            velocity,
            radius,
            max_bounces,
            bounces: 0,
        }
    }

    pub fn add_bounce(&mut self) -> bool {
        self.bounces += 1;
        self.bounces <= self.max_bounces
    }

    pub fn reflect(&mut self, normal: Dir2) {
        self.velocity = self.velocity.reflect(normal.as_vec2());
    }
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
