use macroquad::prelude::*;

// Size of tiles when rendered on screen
const TILE_SIZE: f32 = 32.0;

// Size of each tile in your tileset image (in pixels)
const SOURCE_TILE_SIZE: f32 = 16.0;

const ZOOM_SPEED: f32 = 0.3;
const DRAG_THRESHOLD: f32 = 5.0; // Pixels of movement needed to start a drag
const SELECTED_TILE_ZOOM: f32 = 8.0; // Zoom for the tile preview

#[derive(Clone)]
struct Tile {
    id: usize,
}

struct TileMap {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
    tileset: Texture2D,
}

impl TileMap {
    async fn new() -> Self {
        // Load tileset texture - will panic if file doesn't exist
        let tileset = load_texture("tileset.png").await.unwrap();
        tileset.set_filter(FilterMode::Nearest);

        let width = 16;
        let height = 16;
        let mut tiles = vec![vec![Tile { id: 0 }; width]; height];

        // Add some variety to the map
        for y in 0..height {
            for x in 0..width {
                tiles[y][x] = Tile { id: x + y * width };
            }
        }

        Self {
            tiles,
            width,
            height,
            tileset,
        }
    }

    fn draw(&self, selected_tile: Option<(usize, usize)>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &self.tiles[y][x];

                // Calculate source rectangle based on actual tileset dimensions
                let tiles_per_row = (self.tileset.width() / SOURCE_TILE_SIZE).floor() as f32;
                let src_x = (tile.id as f32 % tiles_per_row) * SOURCE_TILE_SIZE;
                let src_y = (tile.id as f32 / tiles_per_row).floor() * SOURCE_TILE_SIZE;

                // Determine if this is the selected tile
                let is_selected =
                    selected_tile.map_or(false, |(sel_x, sel_y)| x == sel_x && y == sel_y);

                // Draw the tile with other color if selected, otherwise WHITE
                let color = if is_selected { MAGENTA } else { WHITE };

                // Draw the tile
                draw_texture_ex(
                    &self.tileset,
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    color,
                    DrawTextureParams {
                        source: Some(Rect::new(src_x, src_y, SOURCE_TILE_SIZE, SOURCE_TILE_SIZE)),
                        dest_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                );

                // If this is the selected tile, draw a highlight border
                if is_selected {
                    draw_rectangle_lines(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        3.0,
                        RED,
                    );
                }
            }
        }
    }

    // Draw the selected tile preview in the upper right corner
    fn draw_selected_tile_preview(&self, selected_tile: Option<(usize, usize)>) {
        if let Some((sel_x, sel_y)) = selected_tile {
            let tile_id = self.tiles[sel_y][sel_x].id;

            // Switch to screen space
            set_default_camera();

            // Calculate source rectangle
            let tiles_per_row = (self.tileset.width() / SOURCE_TILE_SIZE).floor() as f32;
            let src_x = (tile_id as f32 % tiles_per_row) * SOURCE_TILE_SIZE;
            let src_y = (tile_id as f32 / tiles_per_row).floor() * SOURCE_TILE_SIZE;

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
                &format!("Tile ID: {}", tile_id),
                pos_x,
                pos_y + preview_size + 20.0,
                20.0,
                WHITE,
            );
        }
    }

    // Get tile coordinates under mouse position (in world coordinates)
    fn get_tile_under_mouse(&self, mouse_world_pos: Vec2) -> Option<(usize, usize)> {
        let tile_x = (mouse_world_pos.x / TILE_SIZE).floor() as usize;
        let tile_y = (mouse_world_pos.y / TILE_SIZE).floor() as usize;

        if tile_x < self.width && tile_y < self.height {
            Some((tile_x, tile_y))
        } else {
            None
        }
    }
}

#[macroquad::main("Tilemap Example")]
async fn main() {
    let tilemap = TileMap::new().await;

    // Camera state
    let mut camera_pos = Vec2::new(
        (tilemap.width as f32 * TILE_SIZE) / 2.0,
        (tilemap.height as f32 * TILE_SIZE) / 2.0,
    );
    let mut zoom = 1.0;
    let camera_speed = 5.0;

    // Mouse drag variables
    let mut is_dragging = false;
    let mut drag_start_position = Vec2::new(0.0, 0.0);

    // Track selected tile
    let mut selected_tile: Option<(usize, usize)> = None;

    // Track whether mouse moved during a click (for drag vs. click detection)
    let mut mouse_moved_during_click = false;

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

        // Mouse click/drag handling
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

        // When mouse is released, handle selection if it wasn't a drag
        if is_mouse_button_released(MouseButton::Left) && !mouse_moved_during_click {
            if let Some((tile_x, tile_y)) = tilemap.get_tile_under_mouse(mouse_world_pos) {
                selected_tile = Some((tile_x, tile_y));
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            is_dragging = false;
        }

        // Zoom controls
        let wheel_y = mouse_wheel().1;
        if wheel_y != 0.0 {
            // Adjust zoom with mouse wheel
            zoom += wheel_y * ZOOM_SPEED;
            // Clamp zoom to reasonable values
            zoom = zoom.clamp(0.2, 5.0);
        }

        // Setup camera
        set_camera(&Camera2D {
            target: camera_pos,
            zoom: Vec2::new(zoom * 2.0 / screen_width(), zoom * 2.0 / screen_height()),
            ..Default::default()
        });

        // Draw tilemap with selected tile
        tilemap.draw(selected_tile);

        // Only highlight hovering tiles when not dragging
        if !is_dragging {
            // Get tile coordinates under mouse
            if let Some((tile_x, tile_y)) = tilemap.get_tile_under_mouse(mouse_world_pos) {
                // Highlight the tile under the mouse
                let tile_id = tilemap.tiles[tile_y][tile_x].id;

                // Draw a rectangle outline around the hovered tile
                draw_rectangle_lines(
                    tile_x as f32 * TILE_SIZE,
                    tile_y as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    2.0,
                    YELLOW,
                );

                // Draw UI text in screen space, not world space
                set_default_camera();
                draw_text(
                    &format!("Hover: ({}, {}) ID: {}", tile_x, tile_y, tile_id),
                    10.0,
                    30.0,
                    20.0,
                    WHITE,
                );
            }
        }

        // Draw selected tile info
        if let Some((sel_x, sel_y)) = selected_tile {
            let tile_id = tilemap.tiles[sel_y][sel_x].id;
            set_default_camera();
            draw_text(
                &format!("Selected: ({}, {}) ID: {}", sel_x, sel_y, tile_id),
                10.0,
                60.0,
                20.0,
                RED,
            );
        }

        // Draw the selected tile preview in upper right corner
        tilemap.draw_selected_tile_preview(selected_tile);

        // Draw instructions in screen space
        set_default_camera();
        draw_text(
            "WASD/Arrows to move, Mouse wheel to zoom, Left-click drag to pan, Left-click to select",
            10.0,
            screen_height() - 30.0,
            20.0,
            WHITE,
        );

        next_frame().await;
    }
}
