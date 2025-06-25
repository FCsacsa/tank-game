use std::time::Duration;

use bevy::{
    ecs::{component::Component, resource::Resource},
    math::Vec2,
};

#[derive(Component, Resource)]
pub struct Config {
    pub inactivity_timeout: Duration,
    pub respawn_delay: Duration,

    pub map_dir: String,
    pub map_paths: Vec<String>,

    // Defaults
    pub tank_radius: f32,
    pub track_max_velocity: Vec2,
    pub track_max_acceleration: Vec2,
    pub turret_max_velocity: f32,
    pub turret_max_acceleration: f32,
    pub shoot_delay: Duration,
    pub bullet_radius: f32,
    pub bullet_speed: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inactivity_timeout: Duration::new(5, 0),
            respawn_delay: Duration::new(5, 0),
            map_dir: "./assets/maps".into(),
            map_paths: vec!["map.jsonc".to_owned()],
            tank_radius: 12.0,
            track_max_velocity: [500.0, 500.0].into(),
            track_max_acceleration: [100.0, 100.0].into(),
            turret_max_velocity: 300.0,
            turret_max_acceleration: 100.0,
            shoot_delay: Duration::new(1, 0),
            bullet_radius: 5.0,
            bullet_speed: 100.0,
        }
    }
}
