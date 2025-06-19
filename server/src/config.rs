use std::time::Duration;

use bevy::{ecs::{component::Component, resource::Resource}, math::Vec2};

#[derive(Component, Resource)]
pub struct Config {
    pub inactivity_timeout: Duration,
    pub tank_width: f32,
    pub max_track_speed: Vec2,
    pub map_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inactivity_timeout: Duration::from_secs(5),
            tank_width: 50.0,
            max_track_speed: [500.0, 500.0].into(),
            map_paths: vec!["assets/maps/map.jsonc".to_owned()],
        }
    }
}
