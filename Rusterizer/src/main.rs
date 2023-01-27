mod helpers;
mod mesh;
mod rendering;
mod structs;
mod texture;

use std::{
    f32::{consts::PI, INFINITY},
    path::Path,
};

use glam::{Mat4, Vec2, Vec3};
use mesh::{Mesh, Model};
use minifb::{Key, Window, WindowOptions};
use rendering::Renderer;
use structs::{Transform, Vertex};
use texture::Texture;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut renderer = Renderer {
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
    };
    let mut color_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut depth_buffer: Vec<f32> = vec![INFINITY; WIDTH * HEIGHT];
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
    let mut model = Model::new();
    model.from_gltf(Path::new("D:/Library/Documents/monkey.gltf"));

    let mut camera_transform = Transform {
        translation: glam::vec3(0.0, 0.0, -1.0),
        rotation: glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.0, 0.0),
        scale: glam::vec3(1.0, 1.0, 1.0),
    };

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Clear screen
        for i in 0..color_buffer.len() {
            color_buffer[i] = 0;
            depth_buffer[i] = INFINITY;
        }

        // Set 3rd vertex to mouse position
        (v2.position.x, v2.position.y) = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
        camera_transform.translation.z += 0.1;
        camera_transform.rotation *= glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.01, 0.0);
        renderer.set_view_matrix(camera_transform.matrix());
        renderer.set_projection_matrix(glam::Mat4::perspective_rh(
            0.5 * PI,
            WIDTH as f32 / HEIGHT as f32,
            0.1,
            1000.0,
        ));

        // Draw the triangle
        renderer.draw_model(
            &model,
            &mut color_buffer,
            &mut depth_buffer,
            WIDTH,
            HEIGHT,
            Some(&tex),
        );
        //draw_triangle_filled(v0, v2, v1, &mut buffer, WIDTH, Some(&tex));
        //draw_triangle_wireframe(v0, v2, v1, &mut buffer, WIDTH, HEIGHT);cargo fmt
        //draw_triangle_filled(v0, v3, v2, &mut buffer, WIDTH, Some(&tex));
        //draw_triangle_wireframe(v0, v3, v2, &mut buffer, WIDTH, HEIGHT);
        
        window
            .update_with_buffer(&color_buffer, WIDTH, HEIGHT)
            .unwrap();
        println!("frame rendered");
    }
}
