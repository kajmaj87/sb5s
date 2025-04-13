use crate::camera::CameraController;
use crate::input::{InputEventProcessor, InputManager};
use crate::textures::TextureAtlasManager;
use crate::utils::draw_text_with_background;
use crate::{TileMap, TilePosition};
use lua_engine::lua_engine::LuaEngine;
use lua_engine::LuaError;
use lua_engine::LuaFunction;
use macroquad::prelude::get_fps;
use parking_lot::Mutex;
use std::sync::Arc;

enum UIComponent {
    Label {
        x: f32,
        y: f32,
        handler: LuaFunction,
    },
    // Placeholder for other components we're not implementing yet
    // These would be converted similarly to Label when needed
    Window {
        label: String,
        children: Vec<UIComponent>,
    },
}

impl UIComponent {
    pub fn draw(&self) {
        match self {
            UIComponent::Label { x, y, handler } => {
                // Call the Lua function to draw the label
                match handler.call::<String>(()) {
                    Ok(value) => draw_text_with_background(&value, *x, *y, macroquad::color::WHITE),
                    Err(e) => eprintln!("Error fetching Label value from Lua: {}", e),
                }
            }
            UIComponent::Window { label: _, children } => {
                // Draw the children
                children.iter().for_each(|child| {
                    child.draw();
                });
            }
        }
    }
}

pub struct LuaUIBindings {
    components: Arc<Mutex<Vec<UIComponent>>>,
}

impl LuaUIBindings {
    pub fn new(
        lua_engine: Arc<Mutex<LuaEngine>>,
        camera: Arc<Mutex<CameraController>>,
        input_manager: Arc<Mutex<InputManager>>,
        input_event_processor: Arc<Mutex<InputEventProcessor>>,
        map: Arc<Mutex<TileMap>>,
        texture_atlas_manager: Arc<Mutex<TextureAtlasManager>>,
    ) -> Self {
        let components = Arc::new(Mutex::new(Vec::new()));
        {
            let lua = &lua_engine.lock().lua;
            let globals = lua.globals();

            // Create all the main tables
            let input = lua.create_table().unwrap();
            let ui = lua.create_table().unwrap();
            let tile = lua.create_table().unwrap();
            let atlas = lua.create_table().unwrap();
            let terrain = lua.create_table().unwrap();

            // Register UI components
            {
                let components = components.clone();
                lua.create_function(move |_, (x, y, handler): (f32, f32, LuaFunction)| {
                    components.lock().push(UIComponent::Label { x, y, handler });
                    Ok(())
                })
                .and_then(|f| ui.set("label", f))
                .unwrap();
            }

            // Register FPS function
            lua.create_function(move |_, ()| Ok(get_fps()))
                .and_then(|f| ui.set("fps", f))
                .unwrap();

            // Register tile hover function
            {
                let camera = camera.clone();
                let input_manager = input_manager.clone();
                lua.create_function(move |_, ()| {
                    let tile = TilePosition::from_world_pos(
                        camera
                            .lock()
                            .screen_to_world(input_manager.lock().get_mouse_position()),
                    );
                    Ok((tile.x, tile.y))
                })
                .and_then(|f| tile.set("hovered", f))
                .unwrap();
            }

            // Register tile lookup function
            {
                let map = map.clone();
                lua.create_function(move |_, (x, y): (i32, i32)| {
                    let binding = map.lock();
                    let tile = binding.get_tile(&TilePosition::new(x, y));
                    match tile {
                        Some(tile) => Ok(Some(tile.id)),
                        None => Ok(None),
                    }
                })
                .and_then(|f| tile.set("at", f))
                .unwrap();
            }

            // Register all input functions
            {
                // Register key shortcuts
                let input_manager_clone = input_manager.clone();
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, (key_combo, handler): (String, LuaFunction)| {
                    let keymap = input_manager_clone.lock().get_keymap().clone();
                    let mut processor = input_event_processor_clone.lock();
                    processor
                        .register_shortcut(&key_combo, handler, &keymap)
                        .map_err(LuaError::external)
                })
                .and_then(|f| input.set("register_shortcut", f))
                .unwrap();

                // Register mouse handlers
                let input_manager_clone = input_manager.clone();
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, (button, handler): (String, LuaFunction)| {
                    let keymap = input_manager_clone.lock().get_keymap().clone();
                    let mut processor = input_event_processor_clone.lock();
                    processor
                        .register_mouse(&button, handler, &keymap)
                        .map_err(LuaError::external)
                })
                .and_then(|f| input.set("register_mouse", f))
                .unwrap();

                // Register drag handlers
                let input_manager_clone = input_manager.clone();
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, (button, handler): (String, LuaFunction)| {
                    let keymap = input_manager_clone.lock().get_keymap().clone();
                    let mut processor = input_event_processor_clone.lock();
                    processor
                        .register_drag(&button, handler, &keymap)
                        .map_err(LuaError::external)
                })
                .and_then(|f| input.set("register_drag", f))
                .unwrap();

                // Register mouse move handler
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, handler: LuaFunction| {
                    let mut processor = input_event_processor_clone.lock();
                    processor
                        .register_mouse_move(handler)
                        .map_err(LuaError::external)
                })
                .and_then(|f| input.set("register_mouse_move", f))
                .unwrap();

                // Register mouse wheel handler
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, handler: LuaFunction| {
                    let mut processor = input_event_processor_clone.lock();
                    processor
                        .register_mouse_wheel(handler)
                        .map_err(LuaError::external)
                })
                .and_then(|f| input.set("register_mouse_wheel", f))
                .unwrap();

                // Unregister by string ID
                let input_manager_clone = input_manager.clone();
                let input_event_processor_clone = input_event_processor.clone();
                lua.create_function(move |_, event_id: String| {
                    let keymap = input_manager_clone.lock().get_keymap().clone();
                    let mut processor = input_event_processor_clone.lock();
                    Ok(processor.unregister(&event_id, &keymap))
                })
                .and_then(|f| input.set("unregister", f))
                .unwrap();

                // Set input table to ui table
                ui.set("input", input).unwrap();
            }

            {
                let atlas_manager = texture_atlas_manager.clone();
                lua.create_async_function(
                    move |_, (path, tile_size, width_in_tiles): (String, f32, u32)| {
                        let atlas_manager = atlas_manager.clone();
                        async move {
                            let mut manager = atlas_manager.lock();
                            match manager.create_atlas(&path, tile_size, width_in_tiles).await {
                                Ok(atlas_id) => Ok(atlas_id),
                                Err(e) => Err(LuaError::external(format!(
                                    "Failed to create atlas: {}",
                                    e
                                ))),
                            }
                        }
                    },
                )
                .and_then(|f| atlas.set("register", f))
                .unwrap();
            }

            // Set all tables to their parent tables
            ui.set("tile", tile).unwrap();
            ui.set("atlas", atlas).unwrap();

            // Set ui table to globals
            globals.set("ui", ui).unwrap();
        }
        Self { components }
    }
    pub fn draw(&self) {
        // Draw the UI
        self.components.lock().iter().for_each(|component| {
            component.draw();
        })
    }
}
