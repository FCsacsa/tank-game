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
    deaths: u32,
    /// Respawn delay
    pub respawn_timer: Option<Duration>,

    // sprite information
    /// Path to the tank sprite.
    pub tank_sprite_path: String,
    /// Path to the turret sprite.
    pub turret_sprite_path: String,
    /// Path to the bullet sprite.
    pub bullet_sprite_path: String,

    // tank properties
    /// Radius of the tank's collision circle.
    pub tank_radius: f32,
    /// Maximum allowed track speed.
    pub track_max_velocity: Vec2,
    /// Maximum allowed track acceleration.
    pub track_max_acceleration: Vec2,
    /// Maximum turret rotation speed.
    pub turret_max_velocity: f32,
    /// Maximum turret acceleration.
    pub turret_max_acceleration: f32,

    // player last input
    /// Track acceleration set by the player's last message.
    pub tracks_acceleration_target: Vec2,
    /// Turret acceleration set by the player's last message.
    pub turret_acceleration_target: f32,

    /// Player's last shoot input.
    pub shoot: bool,
    /// Timer since last shoot.
    pub shoot_timer: Option<Duration>,
    /// Amount of time to wait between shots.
    pub shoot_delay: Duration,

    /// Radius of the collision circle of bullets fired by the player.
    pub bullet_radius: f32,
    /// Speed at which the player fires bullets.
    pub bullet_speed: f32,
    /// Number of times the player's bullets can bounce.
    pub bullet_max_bounces: i8,
}

impl Player {
    /// Spawn a new player in.
    /// This will immediately spawn them a tank.
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
                            ..Default::default()
                        },
                        Transform::from_xyz(0.0, 3.0, 0.0),
                        Sprite::from_image(asset_server.load(turret_sprite_path)),
                    ));
            });
    }

    /// Clear last input given by the player.
    pub fn reset_input(&mut self) {
        self.shoot = false;
        self.tracks_acceleration_target = Default::default();
        self.turret_acceleration_target = 0.0;
    }

    /// The player's tank has died, adjust player accordingly.
    /// 1. clear any input
    /// 2. increase deaths
    /// 3. start respawn countdown
    pub fn death(&mut self) {
        self.reset_input();
        self.deaths += 1;
        self.respawn_timer = Some(Duration::new(0, 0));
    }

    pub fn get_deaths(&self) -> u32 {
        self.deaths
    }
}

/// Holds physics data for a tank.
/// Should only be spawned as a [`ChildOf`](bevy::prelude::ChildOf) a [`Player`].
#[derive(Component, Default)]
#[require(Transform, Sprite)]
pub struct Tank {
    /// Velocities of the two tank treads.
    pub track_velocities: Vec2,
    /// Accelerations of the two tank treads.
    pub track_accelerations: Vec2,
    /// Size of the tank
    pub radius: f32,
    /// Maximum velocity allowed for the tracks.
    pub track_max_velocity: Vec2,
}

#[derive(Component, Default)]
#[require(Transform, Sprite)]
pub struct Turret {
    /// Rotational speed of the turret (in radians per second).
    pub velocity: f32,
    /// Maximum velocity.
    pub max_velocity: f32,
    /// Acceleration of the turret.
    pub acceleration: f32,
}

/// Holds physics data for a bullet.
/// It is also used for querying bullets.
#[derive(Component)]
#[require(Transform, Sprite)]
pub struct Bullet {
    /// Current velocity of the bullet (direction and speed).
    pub velocity: Vec2,
    /// Radius of the bullets collision circle.
    pub radius: f32,
    /// Number of bounces that the bullet will survive.
    max_bounces: i8,
    /// Number of times the bullet has already bounced.
    bounces: i8,
}

impl Bullet {
    /// Create a new bullet.
    /// - `velocity`: speed and direction of the bullet
    /// - `radius`: size of the collision circle
    /// - `max_bounces`: number of bounces allowed
    pub fn new(velocity: Vec2, radius: f32, max_bounces: i8) -> Self {
        Self {
            velocity,
            radius,
            max_bounces,
            bounces: 0,
        }
    }

    /// Adds a single bounce to the bullet.
    ///
    /// Returns `true` if the bullet still lives
    pub fn add_bounce(&mut self) -> bool {
        self.bounces += 1;
        self.bounces <= self.max_bounces
    }

    pub fn reflect(&mut self, normal: Dir2) {
        self.velocity = self.velocity.reflect(normal.as_vec2());
    }
}

/// Signifier for the currently loaded map.
/// All map objects are loaded as a children of this, allowing easy despawning.
/// The [`Transform`] is the origin of the map.
#[derive(Component, Resource)]
#[require(Transform)]
pub struct Map;

/// Data for a wall, should be bundled with a [`Transform`] and a [`Sprite`].
#[derive(Component)]
#[require(Transform, Sprite)]
pub struct Wall {
    /// Normal of the wall.
    pub normal: Dir2,
    /// Direction of the wall.
    ///
    /// Calculated from the start and end point.
    pub direction: Dir2,
    /// Half length of the wall.
    ///
    /// Calculated from the distance of the start and end point.
    pub half_length: f32,
}

/// Marker for the spawn points in a [`Map`].
/// Should be bundled together with a [`Transform`].
#[derive(Component)]
#[require(Transform)]
pub struct Spawn();
