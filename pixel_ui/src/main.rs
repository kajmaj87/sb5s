use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

// Constants
mod config {
    use macroquad::prelude::Color;

    pub const TILE_SIZE: f32 = 32.0;
    pub const SOURCE_TILE_SIZE: f32 = 16.0;
    pub const ZOOM_SPEED: f32 = 1.3;
    pub const ZOOM_MIN: f32 = 0.02;
    pub const ZOOM_MAX: f32 = 5.0;
    pub const DRAG_THRESHOLD: f32 = 5.0;
    pub const SELECTED_TILE_ZOOM: f32 = 8.0;
    pub const FPS_HISTORY_SIZE: usize = 60;
    pub const BENCHMARK_MAP_SIZE: usize = 5;
    pub const CAMERA_SPEED: f32 = 5.0;
    pub const TILE_BUFFER: i32 = 2;
    pub const TEXT_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
    pub const TEXT_FONT_SIZE: f32 = 20.0;
    pub const TEXT_PADDING: f32 = 15.0;
}

mod utils {
    use super::config::*;
    use macroquad::math::f32;
    use macroquad::prelude::*;

    pub fn draw_text_with_background(text: &str, x: f32, y: f32, color: Color) {
        let font_size = TEXT_FONT_SIZE;
        let text_dimensions = measure_text(text, None, font_size as u16, 1.0);
        let padding = TEXT_PADDING;

        // Draw background rectangle with padding
        draw_rectangle(
            x - padding,
            y - text_dimensions.offset_y - padding,
            text_dimensions.width + padding * 2.0,
            text_dimensions.height + padding * 2.0,
            TEXT_BACKGROUND_COLOR,
        );

        // Draw text
        draw_text(text, x, y, font_size, color);
    }
    pub fn draw_text_list(texts: Vec<(String, Color)>, x: f32, y: f32) -> f32 {
        let font_size = TEXT_FONT_SIZE;
        let padding = TEXT_PADDING;

        // Use a consistent line height based on font size rather than measuring each string
        let line_height = font_size + 4.0; // Consistent line height

        // Calculate dimensions for all texts
        let mut max_width: f32 = 0.0;
        let total_height = (texts.len() as f32) * line_height;

        for (text, _) in &texts {
            let dimensions = measure_text(text, None, font_size as u16, 1.0);
            max_width = max_width.max(dimensions.width);
        }

        // Draw background rectangle
        draw_rectangle(
            x - padding,
            y,
            max_width + padding * 2.0,
            total_height + padding,
            TEXT_BACKGROUND_COLOR,
        );

        // Draw all texts with consistent spacing
        let mut current_y = y;
        for (text, color) in texts {
            draw_text(&text, x, current_y + line_height, font_size, color);
            current_y += line_height;
        }

        // Return the final y coordinate
        current_y
    }
}

use crate::utils::*;
use config::*;

#[derive(Clone)]
struct Tile {
    id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TilePosition {
    x: i32,
    y: i32,
}

impl TilePosition {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn from_world_pos(world_pos: Vec2) -> Self {
        let tile_x = (world_pos.x / TILE_SIZE).floor() as i32;
        let tile_y = (world_pos.y / TILE_SIZE).floor() as i32;
        Self {
            x: tile_x,
            y: tile_y,
        }
    }

    fn to_world_pos(&self) -> Vec2 {
        Vec2::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE)
    }
}

struct MapBounds {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl MapBounds {
    fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn expand_to_include(&mut self, pos: &TilePosition) {
        self.min_x = self.min_x.min(pos.x);
        self.min_y = self.min_y.min(pos.y);
        self.max_x = self.max_x.max(pos.x);
        self.max_y = self.max_y.max(pos.y);
    }

    fn as_tuple(&self) -> (i32, i32, i32, i32) {
        (self.min_x, self.min_y, self.max_x, self.max_y)
    }
}

struct TileMap {
    tiles: HashMap<(i32, i32), Tile>,
    tileset: Texture2D,
    visible_tiles_count: usize,
    bounds: MapBounds,
    tiles_per_row: f32,
}

impl TileMap {
    async fn new() -> Self {
        let tileset = load_texture("tileset.png").await.unwrap();
        tileset.set_filter(FilterMode::Nearest);

        let tiles_per_row = (tileset.width() / SOURCE_TILE_SIZE).floor() as f32;
        let width = 16;
        let height = 16;
        let mut tiles = HashMap::new();

        for y in 0..height * BENCHMARK_MAP_SIZE {
            for x in 0..width * BENCHMARK_MAP_SIZE {
                tiles.insert(
                    (x as i32, y as i32),
                    Tile {
                        id: (x + y * width) % 256,
                    },
                );
            }
        }

        let bounds = MapBounds::new(
            0,
            0,
            (width * BENCHMARK_MAP_SIZE - 1) as i32,
            (height * BENCHMARK_MAP_SIZE - 1) as i32,
        );

        Self {
            tiles,
            tileset,
            visible_tiles_count: 0,
            bounds,
            tiles_per_row,
        }
    }

    fn get_visible_range(&self, camera: &CameraController) -> (i32, i32, i32, i32) {
        let visible_world_width = screen_width() / camera.zoom;
        let visible_world_height = screen_height() / camera.zoom;

        let min_tile_x =
            ((camera.position.x - visible_world_width / 2.0) / TILE_SIZE).floor() as i32;
        let min_tile_y =
            ((camera.position.y - visible_world_height / 2.0) / TILE_SIZE).floor() as i32;
        let max_tile_x =
            ((camera.position.x + visible_world_width / 2.0) / TILE_SIZE).ceil() as i32;
        let max_tile_y =
            ((camera.position.y + visible_world_height / 2.0) / TILE_SIZE).ceil() as i32;

        // Clamp to map bounds
        let min_x = min_tile_x.max(self.bounds.min_x).min(self.bounds.max_x);
        let min_y = min_tile_y.max(self.bounds.min_y).min(self.bounds.max_y);
        let max_x = max_tile_x.max(self.bounds.min_x).min(self.bounds.max_x);
        let max_y = max_tile_y.max(self.bounds.min_y).min(self.bounds.max_y);

        (min_x, min_y, max_x, max_y)
    }

    fn draw(&mut self, camera: &CameraController, selected_pos: Option<&TilePosition>) {
        let (min_x, min_y, max_x, max_y) = self.get_visible_range(camera);

        // Skip drawing if nothing is visible
        if max_x < min_x || max_y < min_y {
            self.visible_tiles_count = 0;
            return;
        }

        // Collect visible tiles
        let mut tiles_to_draw = Vec::new();
        for x in (min_x - TILE_BUFFER).max(self.bounds.min_x)
            ..=(max_x + TILE_BUFFER).min(self.bounds.max_x)
        {
            for y in (min_y - TILE_BUFFER).max(self.bounds.min_y)
                ..=(max_y + TILE_BUFFER).min(self.bounds.max_y)
            {
                if let Some(tile) = self.tiles.get(&(x, y)) {
                    tiles_to_draw.push((TilePosition::new(x, y), tile));
                }
            }
        }

        // Sort by ID for better rendering efficiency
        tiles_to_draw.sort_by_key(|(_, tile)| tile.id);
        self.visible_tiles_count = tiles_to_draw.len();

        // Draw tiles
        for (pos, tile) in tiles_to_draw {
            let src_x = (tile.id as f32 % self.tiles_per_row) * SOURCE_TILE_SIZE;
            let src_y = (tile.id as f32 / self.tiles_per_row).floor() * SOURCE_TILE_SIZE;

            let is_selected =
                selected_pos.map_or(false, |sel_pos| pos.x == sel_pos.x && pos.y == sel_pos.y);
            let color = if is_selected { MAGENTA } else { WHITE };

            draw_texture_ex(
                &self.tileset,
                pos.x as f32 * TILE_SIZE,
                pos.y as f32 * TILE_SIZE,
                color,
                DrawTextureParams {
                    source: Some(Rect::new(src_x, src_y, SOURCE_TILE_SIZE, SOURCE_TILE_SIZE)),
                    dest_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );
        }
    }

    fn get_tile(&self, pos: &TilePosition) -> Option<&Tile> {
        self.tiles.get(&(pos.x, pos.y))
    }

    fn place_tile(&mut self, pos: &TilePosition, tile_id: usize) {
        self.tiles.insert((pos.x, pos.y), Tile { id: tile_id });
        self.bounds.expand_to_include(pos);
    }

    fn get_initial_center(&self) -> Vec2 {
        Vec2::new(
            (self.bounds.max_x as f32 + self.bounds.min_x as f32) * TILE_SIZE / 2.0,
            (self.bounds.max_y as f32 + self.bounds.min_y as f32) * TILE_SIZE / 2.0,
        )
    }
}

struct CameraController {
    position: Vec2,
    zoom: f32,
}

impl CameraController {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            zoom: 1.0,
        }
    }

    fn update(&mut self, input: &InputManager) {
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

    fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.get_macroquad_camera().screen_to_world(screen_pos)
    }

    fn apply(&self) {
        set_camera(&self.get_macroquad_camera());
    }
}

struct InputManager {
    mouse_position: Vec2,
    prev_mouse_position: Vec2,
    is_dragging: bool,
    drag_start_position: Vec2,
    mouse_moved_during_click: bool,
    zoom_delta: Option<f32>,
    last_painted_pos: Option<TilePosition>,
}

impl InputManager {
    fn new() -> Self {
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

    fn update(&mut self) {
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

    fn is_direction_pressed(&self) -> bool {
        is_key_down(KeyCode::W)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::S)
            || is_key_down(KeyCode::Down)
            || is_key_down(KeyCode::A)
            || is_key_down(KeyCode::Left)
            || is_key_down(KeyCode::D)
            || is_key_down(KeyCode::Right)
    }

    fn is_up_pressed(&self) -> bool {
        is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)
    }

    fn is_down_pressed(&self) -> bool {
        is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)
    }

    fn is_left_pressed(&self) -> bool {
        is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)
    }

    fn is_right_pressed(&self) -> bool {
        is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)
    }

    fn should_select_tile(&self) -> bool {
        is_mouse_button_released(MouseButton::Left) && !self.mouse_moved_during_click
    }

    fn should_place_tile(&self, selected_pos: Option<&TilePosition>) -> bool {
        is_mouse_button_down(MouseButton::Right) && selected_pos.is_some()
    }

    fn get_drag_delta(&self) -> Option<Vec2> {
        if self.is_dragging {
            Some(self.mouse_position - self.prev_mouse_position)
        } else {
            None
        }
    }

    fn get_zoom_delta(&self) -> Option<f32> {
        self.zoom_delta
    }

    fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    fn can_place_at(&mut self, pos: TilePosition) -> bool {
        if self.last_painted_pos != Some(pos) {
            self.last_painted_pos = Some(pos);
            true
        } else {
            false
        }
    }
}

struct UI {}

impl UI {
    fn new() -> Self {
        Self {}
    }

    fn draw_selected_tile_preview(&self, selected_pos: Option<&TilePosition>, map: &TileMap) {
        if let Some(pos) = selected_pos {
            if let Some(tile) = map.get_tile(pos) {
                let preview_size = TILE_SIZE * SELECTED_TILE_ZOOM;
                let pos_x = screen_width() - preview_size - 20.0;
                let pos_y = 20.0;

                // Background
                draw_rectangle(
                    pos_x - 10.0,
                    pos_y - 10.0,
                    preview_size + 20.0,
                    preview_size + 20.0,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                // Tile image
                let src_x = (tile.id as f32 % map.tiles_per_row) * SOURCE_TILE_SIZE;
                let src_y = (tile.id as f32 / map.tiles_per_row).floor() * SOURCE_TILE_SIZE;

                draw_texture_ex(
                    &map.tileset,
                    pos_x,
                    pos_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(src_x, src_y, SOURCE_TILE_SIZE, SOURCE_TILE_SIZE)),
                        dest_size: Some(Vec2::new(preview_size, preview_size)),
                        ..Default::default()
                    },
                );

                // Border
                draw_rectangle_lines(pos_x, pos_y, preview_size, preview_size, 2.0, RED);

                // Tile info
                draw_text(
                    &format!("Tile ID: {}", tile.id),
                    pos_x,
                    pos_y + preview_size + 20.0,
                    20.0,
                    WHITE,
                );
            }
        }
    }

    fn draw_instructions(&self) {
        draw_text_with_background(
            "WASD/Arrows: move, Mouse wheel: zoom, Left-click drag: pan, Left-click: select, Right-click/drag: place tiles",
            10.0,
            screen_height() - 30.0,
            WHITE,
        );
    }
}

struct DebugWindow {
    enabled: bool,
    fps_history: VecDeque<i32>,
}

impl DebugWindow {
    fn new() -> Self {
        Self {
            enabled: true, // On by default
            fps_history: VecDeque::with_capacity(FPS_HISTORY_SIZE),
        }
    }

    fn update(&mut self) {
        let current_fps = get_fps();
        self.fps_history.push_back(current_fps);
        if self.fps_history.len() > FPS_HISTORY_SIZE {
            self.fps_history.pop_front();
        }
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn draw(
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

        // Draw all debug texts with a single background
        draw_text_list(debug_texts, 20.0, 30.0);
    }

    fn draw_tile_highlight(&self, pos: &TilePosition) {
        if !self.enabled {
            return;
        }

        let world_pos = pos.to_world_pos();
        draw_rectangle_lines(world_pos.x, world_pos.y, TILE_SIZE, TILE_SIZE, 2.0, YELLOW);
    }
}

struct GameState {
    map: TileMap,
    camera: CameraController,
    input: InputManager,
    ui: UI,
    debug: DebugWindow,
    selected_pos: Option<TilePosition>,
}

impl GameState {
    async fn new() -> Self {
        let map = TileMap::new().await;
        let camera = CameraController::new(map.get_initial_center());

        Self {
            map,
            camera,
            input: InputManager::new(),
            ui: UI::new(),
            debug: DebugWindow::new(),
            selected_pos: None,
        }
    }

    fn update(&mut self) {
        self.input.update();
        self.camera.update(&self.input);
        self.debug.update();

        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::D) {
            self.debug.toggle();
        }

        // Convert mouse position to world coordinates
        let mouse_world_pos = self.camera.screen_to_world(self.input.get_mouse_position());
        let hover_pos = TilePosition::from_world_pos(mouse_world_pos);

        // Handle tile selection
        if self.input.should_select_tile() {
            if self.map.get_tile(&hover_pos).is_some() {
                self.selected_pos = Some(hover_pos);
            }
        }

        // Handle tile placement
        if self.input.should_place_tile(self.selected_pos.as_ref()) {
            if self.input.can_place_at(hover_pos) {
                if let Some(selected_pos) = &self.selected_pos {
                    if let Some(selected_tile) = self.map.get_tile(selected_pos) {
                        self.map.place_tile(&hover_pos, selected_tile.id);
                    }
                }
            }
        }
    }

    fn draw(&mut self) {
        clear_background(BLACK);

        // Draw world
        self.camera.apply();
        self.map.draw(&self.camera, self.selected_pos.as_ref());

        // Highlight hovered tile if not dragging (only in debug mode)
        if self.input.get_drag_delta().is_none() {
            let hover_pos = TilePosition::from_world_pos(
                self.camera.screen_to_world(self.input.get_mouse_position()),
            );
            self.debug.draw_tile_highlight(&hover_pos);
        }

        // Draw UI (always visible)
        set_default_camera();
        self.ui.draw_instructions();
        self.ui
            .draw_selected_tile_preview(self.selected_pos.as_ref(), &self.map);

        // Draw debug window if enabled
        self.debug.draw(
            &self.map,
            &self.camera,
            self.selected_pos.as_ref(),
            &self.input,
        );
    }
}

#[macroquad::main("Tilemap Example")]
async fn main() {
    let mut game = GameState::new().await;

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
