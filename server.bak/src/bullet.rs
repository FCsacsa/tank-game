use std::time::Duration;

use bevy::{
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, QueryLens, Res},
    },
    hierarchy::Children,
    math::{Vec2, Vec3},
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};

use crate::{
    map::WallData,
    tank::{self, TankData},
    util::{forget_z, with_z},
};

const DIAMETER: f32 = 10.0;
const DIAMETER_SQUARED: f32 = 100.0;
const DEFAULT_BOUNCES: u8 = 1;
const SHOOT_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Component)]
pub struct Bullet {
    bounces: u8,
    max_bounces: u8,
    speed_dir: Vec2,
}

impl Bullet {
    pub fn spawn(translation: Vec3, speed_dir: Vec2, commands: &mut Commands) {
        commands.spawn((
            Transform::from_translation(translation),
            Bullet {
                bounces: 0,
                max_bounces: DEFAULT_BOUNCES,
                speed_dir,
            },
            Sprite::from_color(
                Color::srgb(1.0, 0.0, 0.0),
                Vec2::from_array([DIAMETER, DIAMETER]),
            ),
        ));
    }

    pub fn shoot(
        mut tanks: Query<(&mut TankData, &Children)>,
        mut transforms: Query<&Transform>,
        mut commands: Commands,
    ) {
        let mut tanks: QueryLens<(&mut TankData, &Transform, &Children)> =
            tanks.join_filtered(&mut transforms);
        for (mut tank, position, turret) in &mut tanks.query() {
            if tank.shoot && tank.shoot_timeout >= SHOOT_TIMEOUT {
                let turret = transforms.get(*turret.first().unwrap()).unwrap();
                let dir = position.rotation * turret.up();
                Self::spawn(
                    position.translation + tank::DIAMETER * dir,
                    tank.bullet_speed * forget_z(dir.into()),
                    &mut commands,
                );
                tank.shoot = false;
                tank.shoot_timeout = Duration::from_secs(0);
            }
        }
    }

    pub fn move_bullets(
        mut transforms: Query<&mut Transform>,
        mut walls: Query<&WallData>,
        mut bullets: Query<(&mut Bullet, Entity)>,
        mut commands: Commands,
        time: Res<Time>,
    ) {
        let mut bullets: QueryLens<(&mut Bullet, &mut Transform, Entity)> =
            bullets.join_filtered(&mut transforms);
        let mut walls: QueryLens<(&WallData, &Transform)> = walls.join_filtered(&mut transforms);

        for (mut data, mut transform, entity) in &mut bullets.query() {
            transform.translation += with_z(data.speed_dir * time.delta_secs(), 0.0);

            let mut correction = Vec3::new(0.0, 0.0, 0.0);
            for (wall, origin) in &walls.query() {
                // truncate normal
                let wall_origin = forget_z(origin.translation);
                let bullet_origin = forget_z(transform.translation);
                // from wikipedia
                let in_wall_dist = (wall_origin - bullet_origin).dot(wall.direction);
                let dist_vec = (wall_origin - bullet_origin) - in_wall_dist * wall.direction;
                let out_wall_dist = dist_vec.length();

                if in_wall_dist.abs() <= wall.half_length + (0.5 * DIAMETER)
                    && out_wall_dist < DIAMETER
                {
                    // bullet too close, push it
                    correction += with_z(wall.normal, 0.0) * (DIAMETER - out_wall_dist);
                    data.speed_dir = data.speed_dir.reflect(wall.normal);
                    data.bounces += 1;
                    if data.bounces > data.max_bounces {
                        commands.entity(entity).despawn();
                    }
                }
            }
            transform.translation += correction;
        }
    }

    pub fn hit() {}
}
