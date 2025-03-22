use crate::camera::CameraController;
use crate::input::InputManager;
use crate::utils::draw_text_with_background;
use crate::{TileMap, TilePosition};
use lua_engine::lua_engine::LuaEngine;
use lua_engine::LuaFunction;
use macroquad::prelude::get_fps;
use std::sync::{Arc, Mutex};

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
            UIComponent::Window { label, children } => {
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
        input: Arc<Mutex<InputManager>>,
        map: Arc<Mutex<TileMap>>,
    ) -> Self {
        let components = Arc::new(Mutex::new(Vec::new()));
        {
            let lua = &lua_engine.lock().unwrap().lua;
            let globals = lua.globals();
            let ui = lua.create_table().unwrap();
            let tile = lua.create_table().unwrap();
            {
                let components = components.clone();
                lua.create_function(move |_, (x, y, handler): (f32, f32, LuaFunction)| {
                    components
                        .lock()
                        .unwrap()
                        .push(UIComponent::Label { x, y, handler });
                    Ok(())
                })
                .and_then(|f| ui.set("label", f))
                .unwrap();
            }
            lua.create_function(move |_, ()| Ok(get_fps()))
                .and_then(|f| ui.set("fps", f))
                .unwrap();
            {
                let camera = camera.clone();
                lua.create_function(move |_, ()| {
                    let tile = TilePosition::from_world_pos(
                        camera
                            .lock()
                            .unwrap()
                            .screen_to_world(input.lock().unwrap().get_mouse_position()),
                    );
                    Ok((tile.x, tile.y))
                })
                .and_then(|f| tile.set("hovered", f))
                .unwrap();
            }
            {
                let map = map.clone();
                lua.create_function(move |_, (x, y): (i32, i32)| {
                    let binding = map.lock().unwrap();
                    let tile = binding.get_tile(&TilePosition::new(x, y));
                    match tile {
                        Some(tile) => Ok(Some(tile.id)),
                        None => Ok(None),
                    }
                })
                .and_then(|f| tile.set("at", f))
                .unwrap();
            }
            ui.set("tile", tile).unwrap();
            globals.set("ui", ui).unwrap();
        }
        Self { components }
    }

    pub fn update(&mut self) {}
    pub fn draw(&self) {
        // Draw the UI
        self.components
            .lock()
            .unwrap()
            .iter()
            .for_each(|component| {
                component.draw();
            })
    }
}
