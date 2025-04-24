use std::f32::{EPSILON, consts::PI};

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
    fn new(normal: Vec2, half_length: f32) -> Self {
        let mut transform = Transform::from_xyz(0.0, 100.0, -EPSILON);
        // TODO: rotate transform according to normal.
        Self {
            origin: transform,
            data: WallData {
                normal,
                direction: Vec2 { x: -normal.y, y: normal.x },
                half_length,
            },
        }
    }
}

#[derive(Serialize)]
pub struct Map {
    walls: Vec<(WallData, Vec3)>,
    sprite_path: String, // assuming uniform wall segments
}

impl Map {
    pub fn setup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
        commands.spawn((
            Wall::new(Vec2 { x: 0.0, y: -1.0 }, 100.0),
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), Vec2::new(200.0, 5.0)),
        ));
    }
}

pub fn wall_collision(
    // time: Res<Time>,
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
            println!("{out_wall_dist}");

            if in_wall_dist.abs() <= wall_data.half_length + (0.5 * DIAMETER) && out_wall_dist < DIAMETER {
                // tank too close, push it
                correction += with_z(wall_data.normal, 0.0) * (DIAMETER - out_wall_dist);
            }
        }
        tank_transform.translation += correction;
    }
}
