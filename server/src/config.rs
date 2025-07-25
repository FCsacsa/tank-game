use std::time::Duration;

use bevy::{
    ecs::{component::Component, resource::Resource},
    math::Vec2,
};
use serde::Deserialize;

/// Aggregate `struct` that holds the configuration for the server.
/// This also includes the defaults for players.
#[derive(Component, Resource, Deserialize)]
pub struct Config {
    /// Timeout for player inactivity.
    /// If the player does not send a message within this delay, they will be despawned and
    /// penalised.
    pub inactivity_timeout: Duration,
    /// Timeout after death.
    /// The player is respawned after this much time has passed.
    /// This is shared between players.
    pub respawn_delay: Duration,

    /// Base directory for the map files.
    pub map_dir: String,
    /// Specific paths within `map_dir` that should be loaded.
    pub map_paths: Vec<String>,

    // Defaults for players
    /// Default radius of the tank.
    pub tank_radius: f32,
    /// Default maximum velocity of the two tracks.
    /// The two parts of the vector should have the same (positive) value.
    pub track_max_velocity: Vec2,
    /// Default maximum acceleration for the tank tracks.
    /// The two parts of the vector should have the same (positive) value.
    pub track_max_acceleration: Vec2,
    /// Default for the maximum rotational speed of the tank turret.
    pub turret_max_velocity: f32,
    /// Default for the maximum acceleration of the tank turret.
    pub turret_max_acceleration: f32,
    /// Default delay between the shots fired by a tank.
    pub shoot_delay: Duration,
    /// Default radius of the bullets shot by a tank.
    pub bullet_radius: f32,
    /// Default speed of the bullets fired.
    pub bullet_speed: f32,
    /// Default for the number of bounces that a bullet survives.
    pub bullet_max_bounces: i8,

    pub physics_steps: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inactivity_timeout: Duration::new(5, 0),
            respawn_delay: Duration::new(5, 0),
            map_dir: "./assets/maps".into(),
            map_paths: vec!["layout-00.jsonc".to_owned()],
            tank_radius: 12.0,
            track_max_velocity: [500.0, 500.0].into(),
            track_max_acceleration: [100.0, 100.0].into(),
            turret_max_velocity: 300.0,
            turret_max_acceleration: 100.0,
            shoot_delay: Duration::new(1, 0),
            bullet_radius: 5.0,
            bullet_speed: 100.0,
            bullet_max_bounces: 2,
            physics_steps: 8,
        }
    }
}
