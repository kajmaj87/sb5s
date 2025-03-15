use macroquad::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
// Added for FPS history

// Size of tiles when rendered on screen
const TILE_SIZE: f32 = 32.0;

// Size of each tile in your tileset image (in pixels)
const SOURCE_TILE_SIZE: f32 = 16.0;

const ZOOM_SPEED: f32 = 1.3;
const ZOOM_MIN: f32 = 0.02;
const ZOOM_MAX: f32 = 10.0;
const DRAG_THRESHOLD: f32 = 5.0; // Pixels of movement needed to start a drag
const SELECTED_TILE_ZOOM: f32 = 8.0; // Zoom for the tile preview

// Number of frames to average for the FPS display
const FPS_HISTORY_SIZE: usize = 60; // 60 frames = 1 second at 60 FPS
const BENCHMARK_MAP_SIZE: usize = 20; // Size of the map for benchmarking
#[derive(Clone)]
struct Tile {
    id: usize,
}

struct TileMap {
    // Use a HashMap to store tiles at arbitrary positions
    tiles: HashMap<(i32, i32), Tile>,
    tileset: Texture2D,

    // Keep track of the initial dimensions for camera setup
    initial_width: usize,
    initial_height: usize,
}

impl TileMap {
    async fn new() -> Self {
        // Load tileset texture - will panic if file doesn't exist
        let tileset = load_texture("tileset.png").await.unwrap();
        tileset.set_filter(FilterMode::Nearest);

        let width = 16;
        let height = 16;
        let mut tiles = HashMap::new();

        // Add some variety to the map
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

        Self {
            tiles,
            tileset,
            initial_width: width,
            initial_height: height,
        }
    }
    fn draw(&self, selected_pos: Option<(i32, i32)>, camera_pos: Vec2, zoom: f32) {
        // Calculate the visible area in world coordinates
        let visible_world_width = screen_width() / zoom;
        let visible_world_height = screen_height() / zoom;

        // Calculate the visible area in tile coordinates
        let min_tile_x = ((camera_pos.x - visible_world_width / 2.0) / TILE_SIZE).floor() as i32;
        let min_tile_y = ((camera_pos.y - visible_world_height / 2.0) / TILE_SIZE).floor() as i32;
        let max_tile_x = ((camera_pos.x + visible_world_width / 2.0) / TILE_SIZE).ceil() as i32;
        let max_tile_y = ((camera_pos.y + visible_world_height / 2.0) / TILE_SIZE).ceil() as i32;

        // Add a buffer around the visible area to prevent pop-in when moving
        let buffer = 2;

        // Count how many tiles we're drawing for diagnostics
        let mut tiles_drawn = 0;

        // Only draw tiles that are within the visible area (plus buffer)
        for x in (min_tile_x - buffer)..=(max_tile_x + buffer) {
            for y in (min_tile_y - buffer)..=(max_tile_y + buffer) {
                if let Some(tile) = self.tiles.get(&(x, y)) {
                    tiles_drawn += 1;

                    // Calculate source rectangle based on actual tileset dimensions
                    let tiles_per_row = (self.tileset.width() / SOURCE_TILE_SIZE).floor() as f32;
                    let src_x = (tile.id as f32 % tiles_per_row) * SOURCE_TILE_SIZE;
                    let src_y = (tile.id as f32 / tiles_per_row).floor() * SOURCE_TILE_SIZE;

                    // Determine if this is the selected tile
                    let is_selected =
                        selected_pos.map_or(false, |(sel_x, sel_y)| x == sel_x && y == sel_y);
                    let color = if is_selected { MAGENTA } else { WHITE };

                    // Draw the tile
                    draw_texture_ex(
                        &self.tileset,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        color,
                        DrawTextureParams {
                            source: Some(Rect::new(
                                src_x,
                                src_y,
                                SOURCE_TILE_SIZE,
                                SOURCE_TILE_SIZE,
                            )),
                            dest_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // Display culling statistics
        set_default_camera();
        draw_text(
            &format!(
                "Visible tiles: {}/{} ({:.1}%)",
                tiles_drawn,
                self.tiles.len(),
                100.0 * tiles_drawn as f32 / self.tiles.len() as f32
            ),
            10.0,
            90.0,
            20.0,
            BLUE,
        );
    }

    // Draw the selected tile preview in the upper right corner
    fn draw_selected_tile_preview(&self, selected_pos: Option<(i32, i32)>) {
        if let Some(pos) = selected_pos {
            if let Some(tile) = self.tiles.get(&pos) {
                // Switch to screen space
                set_default_camera();

                // Calculate source rectangle
                let tiles_per_row = (self.tileset.width() / SOURCE_TILE_SIZE).floor() as f32;
                let src_x = (tile.id as f32 % tiles_per_row) * SOURCE_TILE_SIZE;
                let src_y = (tile.id as f32 / tiles_per_row).floor() * SOURCE_TILE_SIZE;

                // Calculate position in upper right corner
                let preview_size = TILE_SIZE * SELECTED_TILE_ZOOM;
                let pos_x = screen_width() - preview_size - 20.0;
                let pos_y = 20.0;

                // Draw background
                draw_rectangle(
                    pos_x - 10.0,
                    pos_y - 10.0,
                    preview_size + 20.0,
                    preview_size + 20.0,
                    Color::new(0.0, 0.0, 0.0, 0.7),
                );

                // Draw the enlarged tile
                draw_texture_ex(
                    &self.tileset,
                    pos_x,
                    pos_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(src_x, src_y, SOURCE_TILE_SIZE, SOURCE_TILE_SIZE)),
                        dest_size: Some(Vec2::new(preview_size, preview_size)),
                        ..Default::default()
                    },
                );

                // Draw a border
                draw_rectangle_lines(pos_x, pos_y, preview_size, preview_size, 2.0, RED);

                // Draw tile info below the preview
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

    // Get map coordinates under mouse position
    fn get_tile_coords_at(&self, world_pos: Vec2) -> (i32, i32) {
        let tile_x = (world_pos.x / TILE_SIZE).floor() as i32;
        let tile_y = (world_pos.y / TILE_SIZE).floor() as i32;
        (tile_x, tile_y)
    }

    // Place a tile at the given coordinates
    fn place_tile(&mut self, pos: (i32, i32), tile_id: usize) {
        self.tiles.insert(pos, Tile { id: tile_id });
    }
}

#[macroquad::main("Tilemap Example")]
async fn main() {
    let mut tilemap = TileMap::new().await;

    // Camera state
    let mut camera_pos = Vec2::new(
        (tilemap.initial_width as f32 * TILE_SIZE) / 2.0,
        (tilemap.initial_height as f32 * TILE_SIZE) / 2.0,
    );
    let mut zoom = 1.0;
    let camera_speed = 5.0;

    // Mouse drag variables
    let mut is_dragging = false;
    let mut drag_start_position = Vec2::new(0.0, 0.0);

    // Track selected tile position
    let mut selected_pos: Option<(i32, i32)> = None;

    // Track whether mouse moved during a click (for drag vs. click detection)
    let mut mouse_moved_during_click = false;

    // Track the last tile position where we placed a tile during drag-painting
    // This helps avoid placing the same tile multiple times
    let mut last_painted_pos: Option<(i32, i32)> = None;

    // For tracking FPS history to calculate average
    let mut fps_history: VecDeque<i32> = VecDeque::with_capacity(FPS_HISTORY_SIZE);

    loop {
        clear_background(BLACK);

        // Camera controls using keyboard
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            camera_pos.y -= camera_speed / zoom;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            camera_pos.y += camera_speed / zoom;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            camera_pos.x -= camera_speed / zoom;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            camera_pos.x += camera_speed / zoom;
        }

        // Mouse click/drag handling (Left button)
        if is_mouse_button_pressed(MouseButton::Left) {
            // When mouse is first pressed, we don't know if it's a drag or click yet
            drag_start_position = Vec2::new(mouse_position().0, mouse_position().1);
            is_dragging = false;
            mouse_moved_during_click = false;
        }

        // While button is held, check if it's a drag
        if is_mouse_button_down(MouseButton::Left) {
            let current_mouse_position = Vec2::new(mouse_position().0, mouse_position().1);
            let delta = current_mouse_position - drag_start_position;
            let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();

            // If mouse moved beyond threshold, it's a drag
            if distance > DRAG_THRESHOLD {
                is_dragging = true;
                mouse_moved_during_click = true;

                // Move camera for drag
                camera_pos.x -= delta.x / zoom;
                camera_pos.y -= delta.y / zoom;

                // Update drag start position for continuous dragging
                drag_start_position = current_mouse_position;
            }
        }

        // Get mouse position in world coordinates
        let mouse_screen_pos = mouse_position();
        let mouse_world_pos = {
            // Get the current camera
            let camera = Camera2D {
                target: camera_pos,
                zoom: Vec2::new(zoom * 2.0 / screen_width(), zoom * 2.0 / screen_height()),
                ..Default::default()
            };

            // Convert screen coordinates to world coordinates
            let screen_pos = Vec2::new(mouse_screen_pos.0, mouse_screen_pos.1);
            camera.screen_to_world(screen_pos)
        };

        // When left mouse is released, handle selection if it wasn't a drag
        if is_mouse_button_released(MouseButton::Left) && !mouse_moved_during_click {
            let tile_coords = tilemap.get_tile_coords_at(mouse_world_pos);
            if tilemap.tiles.contains_key(&tile_coords) {
                selected_pos = Some(tile_coords);
            }
        }

        // Reset last painted position when right mouse button is released
        if is_mouse_button_released(MouseButton::Right) {
            last_painted_pos = None;
        }

        // Right mouse button: place selected tile (now handles continuous painting)
        if is_mouse_button_down(MouseButton::Right) && selected_pos.is_some() {
            let tile_coords = tilemap.get_tile_coords_at(mouse_world_pos);

            // Only place a tile if we haven't placed one at this position yet
            // or if we're at a different position than last frame
            if last_painted_pos != Some(tile_coords) {
                if let Some(selected_coords) = selected_pos {
                    if let Some(selected_tile) = tilemap.tiles.get(&selected_coords) {
                        let tile_id = selected_tile.id;
                        tilemap.place_tile(tile_coords, tile_id);
                        last_painted_pos = Some(tile_coords);
                    }
                }
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            is_dragging = false;
        }

        // Zoom controls
        let wheel_y = mouse_wheel().1;
        if wheel_y != 0.0 {
            // Get the mouse position in world coordinates before zooming
            let mouse_screen_pos = mouse_position();
            let mouse_world_pos_before = {
                let camera = Camera2D {
                    target: camera_pos,
                    zoom: Vec2::new(zoom * 2.0 / screen_width(), zoom * 2.0 / screen_height()),
                    ..Default::default()
                };
                let screen_pos = Vec2::new(mouse_screen_pos.0, mouse_screen_pos.1);
                camera.screen_to_world(screen_pos)
            };

            // Apply zoom change
            if wheel_y > 0.0 {
                zoom /= ZOOM_SPEED;
            } else {
                zoom *= ZOOM_SPEED;
            }
            println!("Zoom: {}, wheel_y {}", zoom, wheel_y);
            // Clamp zoom to reasonable values
            zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);

            // Get the mouse position in world coordinates after zooming
            let mouse_world_pos_after = {
                let camera = Camera2D {
                    target: camera_pos,
                    zoom: Vec2::new(zoom * 2.0 / screen_width(), zoom * 2.0 / screen_height()),
                    ..Default::default()
                };
                let screen_pos = Vec2::new(mouse_screen_pos.0, mouse_screen_pos.1);
                camera.screen_to_world(screen_pos)
            };

            // Adjust camera position to keep the world position under the cursor
            camera_pos.x += mouse_world_pos_before.x - mouse_world_pos_after.x;
            camera_pos.y += mouse_world_pos_before.y - mouse_world_pos_after.y;
        }

        // Setup camera
        set_camera(&Camera2D {
            target: camera_pos,
            zoom: Vec2::new(zoom * 2.0 / screen_width(), zoom * 2.0 / screen_height()),
            ..Default::default()
        });

        // Draw tilemap with selected tile, passing camera parameters
        tilemap.draw(selected_pos, camera_pos, zoom);
        // Only highlight hovering tiles when not dragging
        if !is_dragging {
            // Get tile coordinates under mouse
            let tile_coords = tilemap.get_tile_coords_at(mouse_world_pos);

            // Draw a rectangle outline around the hovered tile position
            draw_rectangle_lines(
                tile_coords.0 as f32 * TILE_SIZE,
                tile_coords.1 as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                2.0,
                YELLOW,
            );

            // Draw hover info
            set_default_camera();
            if let Some(tile) = tilemap.tiles.get(&tile_coords) {
                draw_text(
                    &format!(
                        "Hover: ({}, {}) ID: {}",
                        tile_coords.0, tile_coords.1, tile.id
                    ),
                    10.0,
                    30.0,
                    20.0,
                    WHITE,
                );
            } else {
                draw_text(
                    &format!("Hover: ({}, {}) [Empty]", tile_coords.0, tile_coords.1),
                    10.0,
                    30.0,
                    20.0,
                    WHITE,
                );
            }
        }

        // Draw selected tile info (if any)
        if let Some(pos) = selected_pos {
            if let Some(tile) = tilemap.tiles.get(&pos) {
                set_default_camera();
                draw_text(
                    &format!("Selected: ({}, {}) ID: {}", pos.0, pos.1, tile.id),
                    10.0,
                    60.0,
                    20.0,
                    RED,
                );
            }
        }

        // Draw the selected tile preview in upper right corner
        tilemap.draw_selected_tile_preview(selected_pos);

        // Draw instructions in screen space
        set_default_camera();
        draw_text(
            "WASD/Arrows: move, Mouse wheel: zoom, Left-click drag: pan, Left-click: select, Right-click/drag: place tiles",
            10.0,
            screen_height() - 30.0,
            20.0,
            WHITE,
        );

        // Update FPS history and calculate average
        let current_fps = get_fps();
        fps_history.push_back(current_fps);
        if fps_history.len() > FPS_HISTORY_SIZE {
            fps_history.pop_front();
        }

        let avg_fps: f32 = fps_history.iter().sum::<i32>() as f32 / fps_history.len() as f32;

        // Draw FPS counter with current and average in bottom right corner
        let fps_text = format!("FPS: {} (Avg: {:.1})", current_fps, avg_fps);
        let fps_text_width = measure_text(&fps_text, None, 20, 1.0).width;
        draw_text(
            &fps_text,
            screen_width() - fps_text_width - 10.0,
            screen_height() - 10.0,
            20.0,
            GREEN,
        );

        next_frame().await;
    }
}
