use std::{
    f32::{EPSILON, consts::PI},
    fs::File,
    path::Path,
};

use bevy::{
    asset::AssetServer,
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        system::{Commands, Query, QueryLens, Res},
    },
    math::{Dir3, Vec2, Vec3},
    render::render_resource::encase::private::Length,
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};
use serde::Serialize;
use thiserror::Error;

use crate::tank;

use super::tank::{DIAMETER, TankData};
use super::util::{forget_z, with_z};

#[derive(Component, Serialize)]
pub struct WallData {
    normal: Vec2,
    direction: Vec2,
    half_length: f32,
}

#[derive(Bundle)]
struct Wall {
    origin: Transform,
    data: WallData,
}

impl Wall {
    fn new(normal: Vec2, half_length: f32, origin: Vec2) -> Self {
        let mut transform = Transform::from_translation(with_z(origin, -EPSILON));
        transform = transform.looking_to(transform.forward(), with_z(normal, 0.0));
        // ensure normalise
        let normal = normal.normalize();
        Self {
            origin: transform,
            data: WallData {
                normal,
                direction: Vec2 {
                    x: -normal.y,
                    y: normal.x,
                },
                half_length,
            },
        }
    }
}

pub struct Map {
    walls: Vec<(Vec2, Vec2)>,
    wall_half_length: f32,
    sprite_path: String, // assuming uniform wall segments
}

#[derive(Debug, Error)]
pub enum MapLoadError {
    #[error("Something went wrong while reading file.")]
    FileError(#[from] std::io::Error),
    #[error("File likely not correct json file.")]
    JsonError(#[from] json::Error),
    #[error("Missing fied '{0}' in json.")]
    MissingField(String),
}

impl Map {
    pub fn setup(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) {
        for (normal, origin) in &self.walls {
            commands.spawn((
                Wall::new(*normal, self.wall_half_length, *origin),
                Sprite::from_image(asset_server.load(self.sprite_path.to_owned())),
            ));
        }
    }

    pub fn load_map_from_path(path: &Path) -> Result<Self, MapLoadError> {
        let map = std::fs::read_to_string(path)?;
        Self::load_map_from_str(&map)
    }

    pub fn load_map_from_str(map: &str) -> Result<Self, MapLoadError> {
        let data = json::parse(map)?;
        Ok(Map {
            walls: data["walls"]
                .members()
                .map(|js| {
                    Ok::<_, MapLoadError>((
                        Vec2 {
                            x: js["normal"][0]
                                .as_f32()
                                .ok_or(MapLoadError::MissingField("walls.i.normal.x".to_owned()))?,
                            y: js["normal"][1]
                                .as_f32()
                                .ok_or(MapLoadError::MissingField("walls.i.normal.y".to_owned()))?,
                        },
                        Vec2 {
                            x: js["origin"][0]
                                .as_f32()
                                .ok_or(MapLoadError::MissingField("walls.i.origin.x".to_owned()))?,
                            y: js["origin"][1]
                                .as_f32()
                                .ok_or(MapLoadError::MissingField("walls.i.origin.y".to_owned()))?,
                        },
                    ))
                })
                .collect::<Result<_, _>>()?,
            wall_half_length: data["wall_half_length"]
                .as_f32()
                .ok_or(MapLoadError::MissingField("wall_half_length".to_owned()))?,
            sprite_path: data["sprite_path"]
                .as_str()
                .ok_or(MapLoadError::MissingField("sprite_path".to_owned()))?
                .to_owned(),
        })
    }
}

pub fn wall_collision(
    mut transforms: Query<&mut Transform>,
    mut tank_data: Query<&TankData>,
    mut wall_data: Query<&WallData>,
) {
    let mut tanks: QueryLens<(&TankData, &mut Transform)> =
        tank_data.join_filtered(&mut transforms);
    let mut walls: QueryLens<(&WallData, &Transform)> = wall_data.join_filtered(&mut transforms);
    for (mut tank_data, mut tank_transform) in &mut tanks.query() {
        let mut correction = Vec3::new(0.0, 0.0, 0.0);
        for (wall_data, wall_transform) in &walls.query() {
            // truncate normal
            let wall_origin = forget_z(wall_transform.translation);
            let tank_origin = forget_z(tank_transform.translation);
            // from wikipedia
            let in_wall_dist = (wall_origin - tank_origin).dot(wall_data.direction);
            let dist_vec = (wall_origin - tank_origin) - in_wall_dist * wall_data.direction;
            let out_wall_dist = dist_vec.length();

            if in_wall_dist.abs() <= wall_data.half_length + (0.5 * DIAMETER)
                && out_wall_dist < DIAMETER
            {
                // tank too close, push it
                correction += with_z(wall_data.normal, 0.0) * (DIAMETER - out_wall_dist);
            }
        }
        tank_transform.translation += correction;
    }
}
