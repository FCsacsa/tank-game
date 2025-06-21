use bevy::{
    color::Color,
    ecs::{
        query::Without,
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    input::{ButtonInput, keyboard::KeyCode},
    math::{Isometry2d, Rot2},
    transform::components::Transform,
};

use crate::{
    entities::{Tank, Wall},
    util::forget_z,
};

pub fn do_debug(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::KeyD)
}
pub fn do_normals(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::KeyN)
}
pub fn do_bounds(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::KeyB)
}

pub fn draw_normals(mut gizmos: Gizmos, walls: Query<(&Wall, &Transform)>) {
    for (wall, transform) in &walls {
        let origin = forget_z(transform.translation);
        gizmos.line_2d(
            origin,
            origin + 25.0 * wall.normal.as_vec2(),
            Color::srgb(
                (wall.normal.x + 1.0) / 2.0,
                (wall.normal.y + 1.0) / 2.0,
                1.0,
            ),
        );
    }
}

pub fn draw_bounds(
    mut gizmos: Gizmos,
    tanks: Query<(&Tank, &Transform), Without<Wall>>,
    walls: Query<(&Wall, &Transform), Without<Tank>>,
) {
    for (tank, transform) in &tanks {
        let origin = forget_z(transform.translation);
        gizmos.circle_2d(
            Isometry2d::new(origin, Rot2::default()),
            tank.radius,
            Color::srgb(0.6, 1.0, 0.6),
        );
    }

    for (wall, transform) in &walls {
        let origin = forget_z(transform.translation);
        gizmos.line_2d(
            origin - wall.direction * wall.half_length,
            origin + wall.direction * wall.half_length,
            Color::srgb(0.6, 1.0, 0.6),
        );
    }
}
