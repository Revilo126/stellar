use fent_derive::Component;
use glam::{Quat, Vec3};

#[allow(dead_code)]
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    #[allow(dead_code)]
    pub const IDENTITY: Self = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
}
