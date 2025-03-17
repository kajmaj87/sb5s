use std::sync::{Arc, RwLock};
mod ui;

use lua_engine::lua_engine::LuaEngine;
use ui::MyApp;

fn main() -> eframe::Result<()> {
    // Create the Lua Engine, exposing the core API to Lua
    let lua_engine = Arc::new(RwLock::new(LuaEngine::new()));

    // Run the UI
    let options = eframe::NativeOptions::default();
    let app = MyApp::new(lua_engine.clone());
    if let Err(err) = lua_engine.write().unwrap().run_script("require('init')") {
        eprintln!("Unable to load init.lua due to lua error: {}", err);
    }
    eframe::run_native(
        "Space Business 5",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
