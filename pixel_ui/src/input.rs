use crate::config::DRAG_THRESHOLD;
use crate::TilePosition;

use macroquad::prelude::*;

pub struct InputManager {
    mouse_position: Vec2,
    prev_mouse_position: Vec2,
    is_dragging: bool,
    drag_start_position: Vec2,
    mouse_moved_during_click: bool,
    zoom_delta: Option<f32>,
    last_painted_pos: Option<TilePosition>,
}

impl InputManager {
    pub(crate) fn new() -> Self {
        let initial_pos = Vec2::new(mouse_position().0, mouse_position().1);
        Self {
            mouse_position: initial_pos,
            prev_mouse_position: initial_pos,
            is_dragging: false,
            drag_start_position: initial_pos,
            mouse_moved_during_click: false,
            zoom_delta: None,
            last_painted_pos: None,
        }
    }

    pub(crate) fn update(&mut self) {
        self.prev_mouse_position = self.mouse_position;
        self.mouse_position = Vec2::new(mouse_position().0, mouse_position().1);
        self.zoom_delta = if mouse_wheel().1 != 0.0 {
            Some(mouse_wheel().1)
        } else {
            None
        };

        // Handle mouse button press/release
        if is_mouse_button_pressed(MouseButton::Left) {
            self.drag_start_position = self.mouse_position;
            self.is_dragging = false;
            self.mouse_moved_during_click = false;
        }

        // Check for drag while mouse is down
        if is_mouse_button_down(MouseButton::Left) {
            if !self.is_dragging {
                let delta = self.mouse_position - self.drag_start_position;
                let distance = delta.length();

                if distance > DRAG_THRESHOLD {
                    self.is_dragging = true;
                    self.mouse_moved_during_click = true;
                }
            }
        } else {
            // If left button is not down, we're not dragging
            self.is_dragging = false;
        }
    }

    pub(crate) fn is_direction_pressed(&self) -> bool {
        is_key_down(KeyCode::W)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::S)
            || is_key_down(KeyCode::Down)
            || is_key_down(KeyCode::A)
            || is_key_down(KeyCode::Left)
            || is_key_down(KeyCode::D)
            || is_key_down(KeyCode::Right)
    }

    pub(crate) fn is_up_pressed(&self) -> bool {
        is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)
    }

    pub(crate) fn is_down_pressed(&self) -> bool {
        is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)
    }

    pub(crate) fn is_left_pressed(&self) -> bool {
        is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)
    }

    pub(crate) fn is_right_pressed(&self) -> bool {
        is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)
    }

    pub(crate) fn should_select_tile(&self) -> bool {
        is_mouse_button_released(MouseButton::Left) && !self.mouse_moved_during_click
    }

    pub(crate) fn should_place_tile(&self, selected_pos: Option<&TilePosition>) -> bool {
        is_mouse_button_down(MouseButton::Right) && selected_pos.is_some()
    }

    pub(crate) fn get_drag_delta(&self) -> Option<Vec2> {
        if self.is_dragging {
            Some(self.mouse_position - self.prev_mouse_position)
        } else {
            None
        }
    }

    pub(crate) fn get_zoom_delta(&self) -> Option<f32> {
        self.zoom_delta
    }

    pub(crate) fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub(crate) fn can_place_at(&mut self, pos: TilePosition) -> bool {
        if self.last_painted_pos != Some(pos) {
            self.last_painted_pos = Some(pos);
            true
        } else {
            false
        }
    }
}
