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
    entities::{Bullet, Spawn, Tank, Wall},
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
pub fn do_spawns(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.pressed(KeyCode::KeyS)
}

pub fn draw_normals(mut gizmos: Gizmos, walls: Query<(&Wall, &Transform)>) {
    for (wall, transform) in &walls {
        let origin = forget_z(transform.translation);
        gizmos.line_2d(
            origin - (25.0 / 6.0) * wall.normal.as_vec2(),
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
    tanks: Query<(&Tank, &Transform), (Without<Wall>, Without<Bullet>)>,
    walls: Query<(&Wall, &Transform), (Without<Tank>, Without<Bullet>)>,
    bullets: Query<(&Bullet, &Transform), (Without<Tank>, Without<Wall>)>,
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
        // let dir = wall.direction.normalize();
        // gizmos.rect_2d(
        //     Isometry2d::new(
        //         origin,
        //         Rot2 {
        //             sin: dir.x,
        //             cos: dir.y,
        //         },
        //     ),
        //     Vec2 {
        //         x: 50.0,
        //         y: 2.0 * wall.half_length,
        //     },
        //     Color::srgba(0.3, 1.0, 0.3, 0.4),
        // );
    }

    for (bullet, transform) in &bullets {
        let origin = forget_z(transform.translation);
        gizmos.circle_2d(
            Isometry2d::new(origin, Rot2::default()),
            bullet.radius,
            Color::srgb(1.0, 0.5, 0.5),
        );
    }
}

pub fn draw_spawns(mut gizmos: Gizmos, spawns: Query<(&Spawn, &Transform)>) {
    for (_, spawn) in spawns {
        let origin = forget_z(spawn.translation);
        gizmos.circle_2d(
            Isometry2d::new(origin, Rot2::default()),
            30.0,
            Color::srgb(0.5, 0.5, 1.0),
        );
    }
}
