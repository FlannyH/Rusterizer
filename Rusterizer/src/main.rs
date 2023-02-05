#![allow(clippy::identity_op, clippy::too_many_arguments, dead_code)]

mod camera;
mod helpers;
mod mesh;
mod rendering;
mod structs;
mod texture;
mod triangle_queue;

use std::{
    collections::HashMap,
    f32::{consts::PI, INFINITY},
    path::Path,
    time::Instant,
};

use camera::Camera;
use glam::Mat4;
use mesh::Model;
use minifb::{Key, Window, WindowOptions};
use rendering::Renderer;
use structs::Transform;
use texture::Material;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut renderer = Renderer {
        projection_matrix: Mat4::IDENTITY,
        view_matrix: Mat4::IDENTITY,
        materials: HashMap::<String, Material>::new(),
    };
    let mut color_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut depth_buffer: Vec<f32> = vec![INFINITY; WIDTH * HEIGHT];
    let mut window =
        Window::new("a", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(1666)));

    // Load mesh
    let mut model = Model::new();
    model.create_from_gltf(Path::new("./assets/miptest2.gltf"), &mut renderer);

    let model_transform = Transform {
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

    //camera.transform.translation = glam::vec3(10.873289, 7.532974, -14.913272);
    //camera.pitch = -0.49;
    //camera.yaw = -3.69;
    camera.update(&window, 0.0);

    // Main loop
    let mut now = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let deltatime = now.elapsed().as_secs_f32();
        now = Instant::now();

        // Clear screen
        for i in 0..color_buffer.len() {
            color_buffer[i] = 0;
            depth_buffer[i] = 0.0;
        }

        // println!(
        //     "{}, {}, {}",
        //     camera.transform.translation, camera.pitch, camera.yaw
        // );
        println!("frametime: {deltatime:.6} ms");

        let perspective_matrix =
            glam::Mat4::perspective_rh(0.4 * PI, WIDTH as f32 / HEIGHT as f32, 0.1, 100.0);

        camera.update(&window, deltatime);
        renderer.set_view_matrix(camera.transform.view_matrix());
        renderer.set_projection_matrix(perspective_matrix);

        // Draw the triangle
        renderer.draw_model(
            &model,
            &model_transform,
            &mut color_buffer,
            &mut depth_buffer,
            WIDTH,
            HEIGHT,
        );

        window
            .update_with_buffer(&color_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
