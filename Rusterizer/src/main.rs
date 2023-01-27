mod helpers;
mod mesh;
mod rendering;
mod structs;
mod texture;

use std::path::Path;

use glam::{Vec2, Vec3};
use mesh::{Mesh, Model};
use minifb::{Key, Window, WindowOptions};
use rendering::{draw_triangle_filled, draw_triangle_wireframe};
use structs::Vertex;
use texture::Texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window =
        Window::new("a", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(1666)));

    // Create triangle vertices
    let v0 = Vertex {
        position: Vec3::new(100., 100., 0.),
        colour: Vec3::new(1., 1., 0.),
        uv: Vec2::new(0., 0.),
        normal: Vec3::new(0., 0., 0.),
        tangent: Vec3::new(0., 0., 0.),
    };
    let v1 = Vertex {
        position: Vec3::new(500., 100., 0.),
        colour: Vec3::new(1., 1., 0.),
        uv: Vec2::new(1., 0.),
        normal: Vec3::new(0., 0., 0.),
        tangent: Vec3::new(0., 0., 0.),
    };
    let mut v2 = Vertex {
        position: Vec3::new(500., 500., 0.),
        colour: Vec3::new(0., 1., 1.),
        uv: Vec2::new(1., 1.),
        normal: Vec3::new(0., 0., 0.),
        tangent: Vec3::new(0., 0., 0.),
    };
    let v3 = Vertex {
        position: Vec3::new(100., 500., 0.),
        colour: Vec3::new(0., 1., 1.),
        uv: Vec2::new(0., 1.),
        normal: Vec3::new(0., 0., 0.),
        tangent: Vec3::new(0., 0., 0.),
    };

    // Load texture
    let tex = Texture::load(Path::new(
        "D:/temp/FlanSoundfontPlayer-RW-main/FlanGUI/test.png",
    ));

    // Load mesh
    let mut mesh = Model::new();
    mesh.from_gltf(Path::new("D:/Library/Documents/monkey.gltf"));

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Clear screen
        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        // Set 3rd vertex to mouse position
        (v2.position.x, v2.position.y) = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();

        // Draw the triangle
        draw_triangle_filled(v0, v2, v1, &mut buffer, WIDTH, Some(&tex));
        draw_triangle_wireframe(v0, v2, v1, &mut buffer, WIDTH, HEIGHT);
        draw_triangle_filled(v0, v3, v2, &mut buffer, WIDTH, Some(&tex));
        draw_triangle_wireframe(v0, v3, v2, &mut buffer, WIDTH, HEIGHT);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
