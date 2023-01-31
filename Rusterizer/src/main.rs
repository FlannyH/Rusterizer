mod camera;
mod helpers;
mod mesh;
mod rendering;
mod structs;
mod texture;

use std::{
    collections::HashMap,
    f32::{consts::PI, INFINITY},
    path::Path,
};

use camera::Camera;
use glam::{Mat4, Vec2, Vec3};
use mesh::{Mesh, Model};
use minifb::{Key, MouseMode, Window, WindowOptions};
use rendering::Renderer;
use structs::{Transform, Vertex};
use texture::Texture;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut renderer = Renderer {
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
        textures: HashMap::<String, Texture>::new(),
    };
    let mut color_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut depth_buffer: Vec<f32> = vec![INFINITY; WIDTH * HEIGHT];
    let mut window =
        Window::new("a", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(1666)));

    // Load texture
    let tex = Texture::load(Path::new(
        "D:/temp/FlanSoundfontPlayer-RW-main/FlanGUI/test.png",
    ));

    // Load mesh
    let mut model = Model::new();
    model.create_from_gltf(Path::new("./assets/summerforest.gltf"), &mut renderer);

    let mut model_transform = Transform {
        translation: glam::vec3(0.0, 0.0, 0.0),
        rotation: glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.0, 0.0),
        scale: glam::vec3(1.0, 1.0, 1.0),
    };

    let mut camera = Camera::new(
        &window,
        Transform {
            translation: glam::vec3(0.0, 0.0, 3.0),
            rotation: glam::quat(0.0, 0.0, 0.0, 1.0),
            scale: glam::vec3(1.0, 1.0, 1.0),
        },
        5.0,
        0.005,
    );

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Clear screen
        for i in 0..color_buffer.len() {
            color_buffer[i] = 0;
            depth_buffer[i] = 0.0;
        }

        let perspective_matrix =
            glam::Mat4::perspective_rh(0.4 * PI, WIDTH as f32 / HEIGHT as f32, 0.1, 100.0);

        camera.update(&window, 0.01666);
        //model_transform.rotation *= glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.01, 0.0);
        renderer.set_view_matrix(camera.transform.view_matrix());
        renderer.set_projection_matrix(perspective_matrix);

        //model_transform.translation.x += 0.001;

        // Draw the triangle
        renderer.draw_model(
            &model,
            &model_transform,
            &mut color_buffer,
            &mut depth_buffer,
            WIDTH,
            HEIGHT,
        );
        //draw_triangle_filled(v0, v2, v1, &mut buffer, WIDTH, Some(&tex));
        //draw_triangle_wireframe(v0, v2, v1, &mut buffer, WIDTH, HEIGHT);cargo fmt
        //draw_triangle_filled(v0, v3, v2, &mut buffer, WIDTH, Some(&tex));
        //draw_triangle_wireframe(v0, v3, v2, &mut buffer, WIDTH, HEIGHT);

        window
            .update_with_buffer(&color_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
