mod camera;
mod console;
mod debug;
mod input;
mod lua_ui_integration;

use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::{fs, thread};

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
    pub const BENCHMARK_MAP_SIZE: usize = 1;
    pub const CAMERA_SPEED: f32 = 5.0;
    pub const TILE_BUFFER: i32 = 2;
    pub const TEXT_BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.7);
    pub const TEXT_FONT_SIZE: f32 = 20.0;
    pub const TEXT_PADDING: f32 = 15.0;
    pub const PERSON_SOURCE_TILE_SIZE: f32 = 32.0;
    pub const PERSON_TILE_SIZE: f32 = 32.0;
    pub const PEOPLE_BENCHMARK_SIZE: usize = 100;
    pub const PEOPLE_BENCHMARK_DISPERSION: i32 = 1;
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

use crate::camera::CameraController;
use crate::console::Console;
use crate::debug::DebugWindow;
use crate::input::InputManager;
use crate::lua_ui_integration::LuaUIBindings;
use crate::utils::*;
use config::*;
use lua_engine::lua_client::LuaClient;
use lua_engine::lua_engine::{LuaCommand, LuaEngine};

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
        let tileset = load_texture("assets/tileset.png").await.unwrap();
        tileset.set_filter(FilterMode::Nearest);

        let tiles_per_row = (tileset.width() / SOURCE_TILE_SIZE).floor();
        let width = 16;
        let height = 16;
        let mut tiles = HashMap::new();

        for y in 0..height * BENCHMARK_MAP_SIZE {
            for x in 0..width * BENCHMARK_MAP_SIZE {
                tiles.insert(
                    (x as i32, y as i32),
                    Tile {
                        id: (x + y * height) % 256,
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
struct Animation {
    frames: Vec<usize>,   // Tile IDs for each frame
    current_frame: usize, // Current frame index
    frame_time: f32,      // Time per frame in seconds
    timer: f32,           // Current timer
}

impl Animation {
    // Constructor now takes total_anim_time instead of frame_time
    fn new(frames: Vec<usize>, total_anim_time: f32) -> Self {
        // Calculate frame time to maintain consistent total animation time
        let frame_time = if frames.is_empty() {
            total_anim_time // Avoid division by zero
        } else {
            total_anim_time / frames.len() as f32
        };

        Self {
            frames,
            current_frame: 0,
            frame_time,
            timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.timer += dt;

        // Advance frame if timer exceeds frame_time
        if self.timer >= self.frame_time && !self.frames.is_empty() {
            self.timer -= self.frame_time;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
        }
    }

    fn get_current_frame(&self) -> usize {
        if self.frames.is_empty() {
            0
        } else {
            self.frames[self.current_frame]
        }
    }
}

// Direction enum for people
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn get_animation_frames(&self, tiles_per_row: i32) -> Vec<usize> {
        // Calculate the starting index and frame count based on direction
        let (start_frame, frame_count) = match self {
            Direction::Right => (0, tiles_per_row.min(8)), // First row, up to 8 frames
            Direction::Left => (tiles_per_row, tiles_per_row.min(8)), // Second row
            Direction::Down => (tiles_per_row * 2, tiles_per_row.min(8)), // Third row
            Direction::Up => (tiles_per_row * 3, tiles_per_row.min(8)), // Fourth row
        };

        // Generate frames sequence with explicit type conversion from i32 to usize
        (start_frame..start_frame + frame_count)
            .map(|i| i as usize)
            .collect()
    }

    // Get direction based on movement vector
    fn from_movement(dx: f32, dy: f32) -> Self {
        // Determine the primary direction based on which delta is larger
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if dy > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            }
        }
    }
}
enum PersonState {
    Idle,
    Moving,
}

struct Person {
    position: Vec2,                    // Current world position
    texture: Texture2D,                // Person texture
    tile_pos: TilePosition,            // Current tile position
    start_pos: Vec2,                   // Starting position for movement
    target_pos: Option<Vec2>,          // Target world position for movement
    target_tile: Option<TilePosition>, // Target tile position
    animation: Animation,              // Current animation
    direction: Direction,              // Facing direction
    state: PersonState,                // Current state
    move_timer: f32,                   // Timer for movement (0.0 to 1.0)
    move_duration: f32,                // How long it takes to move one tile (seconds)
    tiles_per_row: i32,                // Calculated per texture
}

impl Person {
    fn new(tile_x: i32, tile_y: i32, direction: Direction, texture: Texture2D) -> Self {
        let tile_pos = TilePosition::new(tile_x, tile_y);
        let position = tile_pos.to_world_pos() + Vec2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0);

        // Calculate tiles_per_row based on texture width
        let tiles_per_row = (texture.width() / PERSON_SOURCE_TILE_SIZE) as i32;

        // Get animation frames for the initial direction
        let frames = direction.get_animation_frames(tiles_per_row);

        // Create animation with consistent total time
        let animation = Animation::new(frames, 0.6); // 0.6s for a complete walk cycle

        Self {
            position,
            tile_pos,
            texture,
            start_pos: position,
            target_pos: None,
            target_tile: None,
            animation,
            direction,
            state: PersonState::Idle,
            move_timer: 0.0,
            move_duration: 1.0,
            tiles_per_row,
        }
    }

    fn update(&mut self, dt: f32) {
        match self.state {
            PersonState::Idle => {
                // Pick a random direction to move
                if rand::gen_range(0.0, 1.0) < 0.02 {
                    // 2% chance to start moving each frame
                    self.pick_random_direction();
                }
            }
            PersonState::Moving => {
                // Update movement timer
                self.move_timer += dt / self.move_duration;

                if self.move_timer >= 1.0 {
                    // Movement complete - snap to final position
                    if let Some(target) = self.target_pos {
                        self.position = target;
                    }
                    if let Some(target_tile) = self.target_tile {
                        self.tile_pos = target_tile;
                    }

                    // Clear targets and return to idle
                    self.target_pos = None;
                    self.target_tile = None;
                    self.state = PersonState::Idle;
                    self.move_timer = 0.0;
                } else {
                    // Interpolate position using the stored start_pos
                    if let Some(target) = self.target_pos {
                        self.position = self.start_pos.lerp(target, self.move_timer);
                    }
                }
            }
        }
        // Always update animation
        self.animation.update(dt);
    }

    fn set_direction(&mut self, direction: Direction) {
        let frames = direction.get_animation_frames(self.tiles_per_row);
        self.direction = direction;
        self.animation = Animation::new(frames, 0.6); // Same 0.6s total animation time
    }

    fn pick_random_direction(&mut self) {
        // 1. Select a random adjacent tile
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let rand_dir = &directions[rand::gen_range(0, directions.len())];

        // Calculate the new target tile
        let mut new_tile = self.tile_pos;
        match rand_dir {
            Direction::Up => new_tile.y -= 1,
            Direction::Down => new_tile.y += 1,
            Direction::Left => new_tile.x -= 1,
            Direction::Right => new_tile.x += 1,
        }

        // 2. Calculate a random point within the inner 3/4 rectangle of the target tile
        let tile_world_pos = new_tile.to_world_pos();
        let inner_size = TILE_SIZE * 0.75;
        let offset = (TILE_SIZE - inner_size) / 2.0;

        // Generate random position within the inner rectangle
        let random_x = tile_world_pos.x + offset + rand::gen_range(0.0, inner_size);
        let random_y = tile_world_pos.y + offset + rand::gen_range(0.0, inner_size);
        let target_pos = Vec2::new(random_x, random_y);

        // 3. Calculate movement vector for direction determination
        let movement_vector = target_pos - self.position;

        // 4. Set direction based on movement vector rather than randomly
        let movement_direction = Direction::from_movement(movement_vector.x, movement_vector.y);
        self.set_direction(movement_direction);

        // 5. Store current position and start moving
        self.start_pos = self.position;
        self.target_pos = Some(target_pos);
        self.target_tile = Some(new_tile);
        self.state = PersonState::Moving;
        self.move_timer = 0.0;
    }

    fn draw(&self) {
        // Get current frame tile ID
        let tile_id = self.animation.get_current_frame();

        // Calculate source rectangle using the texture's actual tiles_per_row
        let src_x = (tile_id as f32 % self.tiles_per_row as f32) * PERSON_SOURCE_TILE_SIZE;
        let src_y = (tile_id as f32 / self.tiles_per_row as f32).floor() * PERSON_SOURCE_TILE_SIZE;

        // Draw person
        draw_texture_ex(
            &self.texture,
            self.position.x - PERSON_TILE_SIZE / 2.0,
            self.position.y - PERSON_TILE_SIZE / 2.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    src_x,
                    src_y,
                    PERSON_SOURCE_TILE_SIZE,
                    PERSON_SOURCE_TILE_SIZE,
                )),
                dest_size: Some(Vec2::new(PERSON_TILE_SIZE, PERSON_TILE_SIZE)),
                ..Default::default()
            },
        );
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

// Define a UI state enum to track the current mode
#[derive(PartialEq)]
enum UIState {
    TileCreation,
    PeopleCreation,
}

struct GameState {
    map: Arc<Mutex<TileMap>>,
    camera: Arc<Mutex<CameraController>>,
    input: Arc<Mutex<InputManager>>,
    ui: UI,
    debug: DebugWindow,
    selected_pos: Option<TilePosition>,
    people: Vec<Person>,
    last_frame_time: f64,
    ui_state: UIState,
    character_textures: Vec<Texture2D>,
    last_person_pos: Option<Vec2>,
    console: Console,
    lua_client: Arc<LuaClient>,
    lua_ui: LuaUIBindings,
}

impl GameState {
    async fn new(command_tx: Sender<LuaCommand>, lua_engine: Arc<Mutex<LuaEngine>>) -> Self {
        // Create the client that the game state will use
        let lua_client = Arc::new(LuaClient::new(command_tx.clone()));
        let map = Arc::new(Mutex::new(TileMap::new().await));
        let initial_center = { map.lock().unwrap().get_initial_center() };
        let camera = Arc::new(Mutex::new(CameraController::new(initial_center)));
        let input = Arc::new(Mutex::new(InputManager::new()));
        let lua_ui = LuaUIBindings::new(
            lua_engine.clone(),
            camera.clone(),
            input.clone(),
            map.clone(),
        );

        // Load character textures
        let character_paths = find_character_textures("assets");
        let mut character_textures = Vec::new();

        for path in &character_paths {
            if let Some(path_str) = path.to_str() {
                match load_texture(path_str).await {
                    Ok(texture) => {
                        texture.set_filter(FilterMode::Nearest);
                        character_textures.push(texture);
                    }
                    Err(e) => println!("Failed to load texture {}: {:?}", path_str, e),
                }
            }
        }

        // Create initial people
        let mut people = Vec::new();

        for _ in 0..PEOPLE_BENCHMARK_SIZE {
            let tile_x = 1 + rand::gen_range(0, PEOPLE_BENCHMARK_DISPERSION);
            let tile_y = 1 + rand::gen_range(0, PEOPLE_BENCHMARK_DISPERSION);

            if !character_textures.is_empty() {
                // Select random texture
                let texture_index = rand::gen_range(0, character_textures.len());
                let texture = character_textures[texture_index].clone();

                // Random direction
                let direction = match rand::gen_range(0, 4) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    _ => Direction::Right,
                };

                people.push(Person::new(tile_x, tile_y, direction, texture));
            }
        }

        Self {
            map,
            camera: camera.clone(),
            input: input.clone(),
            ui: UI::new(),
            debug: DebugWindow::new(),
            selected_pos: None,
            people,
            last_frame_time: get_time(),
            ui_state: UIState::TileCreation, // Default state
            character_textures,
            last_person_pos: None,
            console: Console::new(lua_client.clone()),
            lua_client,
            lua_ui,
        }
    }

    fn update(&mut self) {
        let current_time = get_time();
        let dt = (current_time - self.last_frame_time) as f32;
        self.last_frame_time = current_time;
        if is_key_pressed(KeyCode::GraveAccent) {
            self.console.toggle();
        }

        // Update people
        for person in &mut self.people {
            person.update(dt);
        }

        // Update input
        {
            let mut input = self.input.lock().unwrap();
            input.update();
        }

        // Update and draw the console
        if self.console.visible {
            self.console.update();
            return;
        }
        // Update camera with input
        {
            let mut camera = self.camera.lock().unwrap();
            let input = self.input.lock().unwrap();
            camera.update(&input);
        }

        self.debug.update();

        // Toggle debug mode
        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::D) {
            self.debug.toggle();
        }

        if is_key_pressed(KeyCode::E) {
            self.ui_state = UIState::PeopleCreation;
        }

        // Convert mouse position to world coordinates
        let mouse_world_pos;
        let hover_pos;
        {
            let camera = self.camera.lock().unwrap();
            let input = self.input.lock().unwrap();
            mouse_world_pos = camera.screen_to_world(input.get_mouse_position());
        }
        hover_pos = TilePosition::from_world_pos(mouse_world_pos);

        // Handle tile selection
        let should_select;
        {
            let input = self.input.lock().unwrap();
            should_select = input.should_select_tile();
        }

        if should_select {
            // Check if tile exists with lock
            let tile_exists = {
                let map = self.map.lock().unwrap();
                map.get_tile(&hover_pos).is_some()
            };

            if tile_exists {
                self.selected_pos = Some(hover_pos);
                self.ui_state = UIState::TileCreation;
            }
        }

        // Handle actions based on UI state
        match self.ui_state {
            UIState::TileCreation => {
                // Check conditions for tile placement
                let should_place_tile;
                let can_place;
                {
                    let mut input = self.input.lock().unwrap();
                    should_place_tile = input.should_place_tile(self.selected_pos.as_ref());
                    can_place = input.can_place_at(hover_pos);
                }

                // Handle tile placement
                if should_place_tile && can_place {
                    if let Some(selected_pos) = &self.selected_pos {
                        // Get the tile ID from the selected position
                        let selected_tile_id = {
                            let map = self.map.lock().unwrap();
                            map.get_tile(selected_pos).map(|tile| tile.id)
                        };

                        // Place the tile if we found a valid ID
                        if let Some(tile_id) = selected_tile_id {
                            let mut map = self.map.lock().unwrap();
                            map.place_tile(&hover_pos, tile_id);
                        }
                    }
                }
            }
            UIState::PeopleCreation => {
                // Handle person creation with dragging - now purely distance-based
                if is_mouse_button_down(MouseButton::Right) {
                    // Check if we've moved enough since last person creation
                    let should_create = match self.last_person_pos {
                        Some(last_pos) => {
                            // Only create if we've moved at least 1/4 of a tile
                            let distance = (mouse_world_pos - last_pos).length();
                            distance > TILE_SIZE * 0.25
                        }
                        None => true, // Always create first person
                    };

                    if should_create {
                        // Add a person directly at mouse position
                        self.add_person_at_position(hover_pos, mouse_world_pos);
                        self.last_person_pos = Some(mouse_world_pos);
                    }
                } else if is_mouse_button_released(MouseButton::Right) {
                    // Reset when mouse button is released
                    self.last_person_pos = None;
                }
            }
        }
    }

    fn add_person_at_position(&mut self, tile_pos: TilePosition, world_pos: Vec2) {
        if !self.character_textures.is_empty() {
            let texture_index = rand::gen_range(0, self.character_textures.len());
            let texture = self.character_textures[texture_index].clone();

            // Random direction
            let random_dir = match rand::gen_range(0, 4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            };

            // Create person and set position directly to mouse position
            let mut person = Person::new(tile_pos.x, tile_pos.y, random_dir, texture);
            person.position = world_pos;

            // Add to people list
            self.people.push(person);
        }
    }

    fn draw(&mut self) {
        clear_background(BLACK);

        // Draw world
        {
            let camera = self.camera.lock().unwrap();
            camera.apply();

            // Draw map with locked access
            {
                let mut map = self.map.lock().unwrap();
                map.draw(&camera, self.selected_pos.as_ref());
            }

            for person in &self.people {
                person.draw(); // Using the updated draw method without tiles_per_row
            }

            // Highlight hovered tile if not dragging (only in debug mode)
            {
                let input = self.input.lock().unwrap();
                if input.get_drag_delta().is_none() {
                    let mouse_pos = input.get_mouse_position();
                    let hover_pos = TilePosition::from_world_pos(camera.screen_to_world(mouse_pos));
                    self.debug.draw_tile_highlight(&hover_pos);
                }
            }
        }

        // Draw UI (always visible)
        set_default_camera();
        self.ui.draw_instructions();

        // Draw tile preview with locked map
        {
            let map = self.map.lock().unwrap();
            self.ui
                .draw_selected_tile_preview(self.selected_pos.as_ref(), &map);
        }

        // Display mode-specific message
        match self.ui_state {
            UIState::PeopleCreation => {
                draw_text_with_background(
                    "PEOPLE CREATION MODE (select a tile to exit, Right-click to add person)",
                    10.0,
                    screen_height() - 60.0,
                    YELLOW,
                );
            }
            UIState::TileCreation => {
                // Optional: Show tile creation mode text
                if self.selected_pos.is_some() {
                    draw_text_with_background(
                        "TILE CREATION MODE (Press `e` to switch to people mode)",
                        10.0,
                        screen_height() - 60.0,
                        GREEN,
                    );
                }
            }
        }

        // Draw debug window if enabled
        {
            let camera = self.camera.lock().unwrap();
            let input = self.input.lock().unwrap();
            let map = self.map.lock().unwrap();
            self.debug
                .draw(&map, &camera, self.selected_pos.as_ref(), &input);
        }

        self.lua_ui.draw();
        // Draw console
        self.console.draw();
    }
}
// Function to find character textures using standard fs
fn find_character_textures(dir_path: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Recursively search for Walk.png files
    visit_dirs(Path::new(dir_path), &mut paths).unwrap_or_else(|e| {
        println!("Error scanning directory: {:?}", e);
    });

    paths
}

// Helper function for recursive directory traversal
fn visit_dirs(dir: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, paths)?;
            } else if let Some(extension) = path.extension() {
                if extension == "png" {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name) = file_name.to_str() {
                            if name.ends_with("Walk.png") && !name.contains("Shadow") {
                                paths.push(path.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[macroquad::main("Tilemap Example")]
async fn main() {
    let (command_tx, command_rx) = mpsc::channel();
    let lua_engine = Arc::new(Mutex::new(LuaEngine::new(command_rx)));
    let mut game = GameState::new(command_tx, lua_engine.clone()).await;
    if let Err(e) = lua_engine.lock().unwrap().run_script(
        r#"-- Add scripts directory to Lua's package path
        package.path = "./scripts/?.lua;" .. package.path
        require('init')"#,
    ) {
        println!("Error during lua initialization: {:?}", e);
    }
    // Create game state with client
    // spawn thread to run the lua engine
    thread::spawn(move || {
        lua_engine.lock().unwrap().run();
    });

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
