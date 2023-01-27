use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub colour: Vec3,
    pub uv: Vec2,
}

pub struct FragIn {
    pub position: Vec4,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub colour: Vec3,
    pub uv: Vec2,
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}
