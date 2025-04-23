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

use super::tank::{DIAMETER_SQUARED, TankData};
use super::util::forget_z;

#[derive(Component, Serialize)]
pub struct WallData {
    normal: Dir3,
    length: f32,
}

#[derive(Bundle)]
struct Wall {
    origin: Transform,
    data: WallData,
}

impl Wall {
    fn new(length: f32) -> Self {
        let mut transform = Transform::from_xyz(0.0, 100.0, -EPSILON);
        transform.rotate_z(PI);
        Self {
            origin: transform,
            data: WallData {
                normal: transform.up(),
                length,
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
            Wall::new(100.0),
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), Vec2::new(100.0, 5.0)),
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
    let mut walls: QueryLens<(&WallData, &Transform)> =
        wall_data.join_filtered(&mut transforms);
    for (mut tank_data, mut tank_transform) in &mut tanks.query() {
        // let mut correction = Vec3::new(0.0, 0.0, 0.0);
        for (mut wall_data, mut wall_origin) in &mut walls.query() {
            // TODO: find line from origin + normal
            let wall_dir = forget_z(wall_data.normal.cross(Vec3::new(0.0, 0.0, 1.0)));
            let c =
                -(wall_dir.x * wall_origin.translation.x + wall_dir.y * wall_origin.translation.y);
            // TODO: use line and tank pos to get closest point and its distances to
            // origin and tank
            let tank_pos = forget_z(tank_transform.translation);
            let closest_point = Vec2::new(
                (wall_dir.y * (wall_dir.y * tank_pos.x - wall_dir.x * tank_pos.y) - wall_dir.x * c)
                    / wall_dir.length_squared(),
                (wall_dir.x * (wall_dir.x * tank_pos.y - wall_dir.y * tank_pos.x) - wall_dir.y * c)
                    / wall_dir.length_squared(),
            );
            let distance = (wall_dir.dot(tank_pos) + c) / wall_dir.length();
            println!("{distance}");
            // NOTE: set z coords of distance to 0
            // TODO: check and move tank if needed
        }
    }
}
