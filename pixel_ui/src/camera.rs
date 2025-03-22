use crate::config::{CAMERA_SPEED, ZOOM_MAX, ZOOM_MIN, ZOOM_SPEED};
use crate::input::InputManager;

use macroquad::prelude::*;

pub struct CameraController {
    pub(crate) position: Vec2,
    pub(crate) zoom: f32,
}

impl CameraController {
    pub(crate) fn new(position: Vec2) -> Self {
        Self {
            position,
            zoom: 1.0,
        }
    }

    pub(crate) fn update(&mut self, input: &InputManager) {
        // Handle keyboard movement
        if input.is_direction_pressed() {
            let move_speed = CAMERA_SPEED / self.zoom;
            if input.is_up_pressed() {
                self.position.y -= move_speed;
            }
            if input.is_down_pressed() {
                self.position.y += move_speed;
            }
            if input.is_left_pressed() {
                self.position.x -= move_speed;
            }
            if input.is_right_pressed() {
                self.position.x += move_speed;
            }
        }

        // Handle drag movement
        if let Some(drag_delta) = input.get_drag_delta() {
            self.position.x -= drag_delta.x / self.zoom;
            self.position.y -= drag_delta.y / self.zoom;
        }

        // Handle zoom
        if let Some(zoom_delta) = input.get_zoom_delta() {
            // Store pre-zoom mouse world position
            let pre_zoom_pos = self.screen_to_world(input.get_mouse_position());

            // Apply zoom
            if zoom_delta < 0.0 {
                self.zoom /= ZOOM_SPEED;
            } else {
                self.zoom *= ZOOM_SPEED;
            }
            self.zoom = self.zoom.clamp(ZOOM_MIN, ZOOM_MAX);

            // Get post-zoom mouse world position
            let post_zoom_pos = self.screen_to_world(input.get_mouse_position());

            // Adjust to keep world position under cursor
            self.position.x += pre_zoom_pos.x - post_zoom_pos.x;
            self.position.y += pre_zoom_pos.y - post_zoom_pos.y;
        }
    }

    fn get_macroquad_camera(&self) -> Camera2D {
        Camera2D {
            target: self.position,
            zoom: Vec2::new(
                self.zoom * 2.0 / screen_width(),
                self.zoom * 2.0 / screen_height(),
            ),
            ..Default::default()
        }
    }

    pub(crate) fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.get_macroquad_camera().screen_to_world(screen_pos)
    }

    pub(crate) fn apply(&self) {
        set_camera(&self.get_macroquad_camera());
    }
}
