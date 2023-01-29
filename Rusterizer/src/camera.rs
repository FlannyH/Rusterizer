use minifb::{Key, MouseButton, MouseMode, Window};

use crate::structs::Transform;

pub struct Camera {
    pub transform: Transform,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    mouse_pos_old: (f32, f32),
    should_skip_mouse_update: bool,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new(
        window: &Window,
        transform: Transform,
        move_speed: f32,
        mouse_sensitivity: f32,
    ) -> Self {
        Camera {
            transform,
            move_speed,
            mouse_sensitivity,
            mouse_pos_old: window.get_mouse_pos(MouseMode::Pass).unwrap(),
            pitch: 0.0,
            yaw: 0.0,
            should_skip_mouse_update: false,
        }
    }
    pub fn update(&mut self, window: &Window, delta_time: f32) {
        // Moving forwards, backwards, left and right
        if window.is_key_down(Key::A) {
            self.transform.translation -= self.move_speed * delta_time * self.transform.right()
        }
        if window.is_key_down(Key::D) {
            self.transform.translation += self.move_speed * delta_time * self.transform.right()
        }
        if window.is_key_down(Key::W) {
            self.transform.translation += self.move_speed * delta_time * self.transform.forward()
        }
        if window.is_key_down(Key::S) {
            self.transform.translation -= self.move_speed * delta_time * self.transform.forward()
        }

        // Moving up and down, Minecraft style
        if window.is_key_down(Key::Space) {
            self.transform.translation += self.move_speed * delta_time * glam::vec3(0.0, 1.0, 0.0);
        }
        if window.is_key_down(Key::LeftShift) {
            self.transform.translation -= self.move_speed * delta_time * glam::vec3(0.0, 1.0, 0.0);
        }

        // Mouse rotation'
        if window.get_mouse_down(MouseButton::Right) {
            // Update mouse position
            let mouse_pos = window.get_mouse_pos(MouseMode::Pass).unwrap();
            let delta_mouse = (
                mouse_pos.0 - self.mouse_pos_old.0,
                mouse_pos.1 - self.mouse_pos_old.1,
            );
            self.mouse_pos_old = mouse_pos;

            // If the mouse position is a specific high value, that means we're still settling in after starting to hold right click
            if !self.should_skip_mouse_update {
                self.pitch -= delta_mouse.1 * self.mouse_sensitivity;
                self.yaw -= delta_mouse.0 * self.mouse_sensitivity;
                self.transform.rotation =
                    glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0)
            } else {
                self.should_skip_mouse_update = false;
            }
        } else {
            self.should_skip_mouse_update = true;
        }
    }
}
