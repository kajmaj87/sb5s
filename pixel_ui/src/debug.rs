use crate::camera::CameraController;
use crate::config::{FPS_HISTORY_SIZE, TILE_SIZE};
use crate::input::InputManager;
use crate::utils::draw_text_list;
use crate::{TileMap, TilePosition};
use macroquad::prelude::*;
use std::collections::VecDeque;

pub struct DebugWindow {
    enabled: bool,
    fps_history: VecDeque<i32>,
}

impl DebugWindow {
    pub(crate) fn new() -> Self {
        Self {
            enabled: true, // On by default
            fps_history: VecDeque::with_capacity(FPS_HISTORY_SIZE),
        }
    }

    pub(crate) fn update(&mut self) {
        let current_fps = get_fps();
        self.fps_history.push_back(current_fps);
        if self.fps_history.len() > FPS_HISTORY_SIZE {
            self.fps_history.pop_front();
        }
    }

    pub(crate) fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub(crate) fn draw(
        &self,
        map: &TileMap,
        camera: &CameraController,
        selected_pos: Option<&TilePosition>,
        input: &InputManager,
    ) {
        if !self.enabled {
            return;
        }

        let mut debug_texts = Vec::new();

        // Add hover info if not dragging
        if input.get_drag_delta().is_none() {
            let hover_pos =
                TilePosition::from_world_pos(camera.screen_to_world(input.get_mouse_position()));

            if let Some(tile) = map.get_tile(&hover_pos) {
                debug_texts.push((
                    format!("Hover: ({}, {}) ID: {}", hover_pos.x, hover_pos.y, tile.id),
                    WHITE,
                ));
            } else {
                debug_texts.push((
                    format!("Hover: ({}, {}) [Empty]", hover_pos.x, hover_pos.y),
                    WHITE,
                ));
            }
        }

        // Add selected tile info
        if let Some(pos) = selected_pos {
            if let Some(tile) = map.get_tile(pos) {
                debug_texts.push((
                    format!("Selected: ({}, {}) ID: {}", pos.x, pos.y, tile.id),
                    RED,
                ));
            }
        }

        // Add statistics
        debug_texts.push((
            format!(
                "Visible tiles: {}/{} ({:.1}%)",
                map.visible_tiles_count,
                map.tiles.len(),
                100.0 * map.visible_tiles_count as f32 / map.tiles.len() as f32
            ),
            BLUE,
        ));

        debug_texts.push((format!("Zoom: {:.3}", camera.zoom), SKYBLUE));

        let bounds = map.bounds.as_tuple();
        debug_texts.push((
            format!(
                "Map bounds: ({}, {}) to ({}, {})",
                bounds.0, bounds.1, bounds.2, bounds.3
            ),
            GREEN,
        ));

        let avg_fps: f32 =
            self.fps_history.iter().sum::<i32>() as f32 / self.fps_history.len().max(1) as f32;
        debug_texts.push((format!("FPS: {} (Avg: {:.1})", get_fps(), avg_fps), GREEN));
        debug_texts.push((
            "Shift+D to toggle debug mode window, ` (accent) to open console"
                .parse()
                .unwrap(),
            WHITE,
        ));

        // Draw all debug texts with a single background
        draw_text_list(debug_texts, 20.0, 30.0);
    }

    pub(crate) fn draw_tile_highlight(&self, pos: &TilePosition) {
        if !self.enabled {
            return;
        }

        let world_pos = pos.to_world_pos();
        draw_rectangle_lines(world_pos.x, world_pos.y, TILE_SIZE, TILE_SIZE, 2.0, YELLOW);
    }
}
