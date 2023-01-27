use glam::{Vec2, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub colour: Vec3,
    pub uv: Vec2,
}
