use bevy::math::{Vec2, Vec3};

pub fn forget_z(vec: Vec3) -> Vec2 {
    Vec2::new(vec.x, vec.y)
}
