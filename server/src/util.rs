use bevy::math::{Vec2, Vec3};

pub fn forget_z(vec: Vec3) -> Vec2 {
    Vec2::new(vec.x, vec.y)
}

pub fn with_z(vec: Vec2, z: f32) -> Vec3 {
    Vec3 {
        x: vec.x,
        y: vec.y,
        z,
    }
}
