use bevy::{
    asset::AssetServer, ecs::{component::Component, resource::Resource, system::{Commands, Res}}, log, math::{Dir2, Vec2}, sprite::Sprite, transform::components::Transform
};
use serde::Deserialize;

use crate::{
    entities::{self, Spawn},
    util::{with_z, with_z_arr},
};

#[derive(Deserialize)]
struct Wall {
    from: [f32; 2],
    to: [f32; 2],
    normal: [f32; 2],
}

#[derive(Deserialize)]
pub struct Map {
    name: String,
    background_path: String,
    walls: Vec<Wall>,
    spawns: Vec<[f32; 2]>,
}

impl Map {
    pub fn spawn(&self, mut commands: Commands, asset_server: Res<AssetServer>) {
        commands
            .spawn((
                entities::Map {},
                Sprite::from_image(asset_server.load(&self.background_path)),
            ))
            .with_children(|parent| {
                for wall in &self.walls {
                    let from: Vec2 = wall.from.into();
                    let to: Vec2 = wall.to.into();
                    let position = with_z((from + to) / 2.0, 0.0);
                    if let Ok((direction, length)) = Dir2::new_and_length(to - from) {
                        let normal = Dir2::from_xy(wall.normal[0], wall.normal[1]).unwrap_or(
                            Dir2::new_unchecked(direction.rotate(Vec2 { x: 0.0, y: 1.0 })),
                        );
                        parent.spawn((
                            entities::Wall {
                                normal,
                                direction,
                                half_length: length / 2.0,
                            },
                            Transform::from_translation(position),
                        ));
                    } else {
                        log::warn!("The current map \"{}\" contains 0 length walls.", self.name);
                    }
                }
                for &spawn in &self.spawns {
                    let pos = with_z_arr(spawn, 0.0);
                    parent.spawn((Spawn(), Transform::from_translation(pos)));
                }
            });
    }
}

#[derive(Component, Resource)]
pub struct Maps {
    pub(crate) loaded: Vec<Map>,
    pub(crate) current: Option<usize>,
}
