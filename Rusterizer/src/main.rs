use glam::Vec2;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub fn to_argb8(a: u8, r: u8, g: u8, b: u8) -> u32 {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

fn edge_function(v0: Vec2, v1: Vec2, p: Vec2) -> f32 {
    let v0_p = p - v0;
    let v0_v1 = v1 - v0;
    (v0_p.x * v0_v1.y) - (v0_p.y * v0_v1.x)
}

fn point_inside_triangle(v0: Vec2, v1: Vec2, v2: Vec2, p: Vec2) -> bool {
    (edge_function(v0, v1, p) < 0.0)
        && (edge_function(v1, v2, p) < 0.0)
        && (edge_function(v2, v0, p) < 0.0)
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window =
        Window::new("a", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

    // Create triangle vertices
    let v0 = Vec2::new(100., 100.);
    let v1 = Vec2::new(500., 500.);
    let mut v2 = Vec2::new(100., 500.);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in 0..buffer.len() {
            let x = i % WIDTH;
            let y = i / WIDTH;
            let p = Vec2::new(x as f32, y as f32);
            (v2.x, v2.y) = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
            //buffer[i] = to_argb8(255u8, (x * 255 / WIDTH) as u8, (y * 255 / HEIGHT) as u8, 0u8)
            let inside =
                point_inside_triangle(v0, v1, v2, p) | point_inside_triangle(v0, v2, v1, p);
            if inside {
                buffer[i] = to_argb8(255u8, 255, 255, 255);
            } else {
                buffer[i] = to_argb8(255u8, 0, 0, 0);
            }
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
