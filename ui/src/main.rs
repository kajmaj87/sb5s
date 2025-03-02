use std::sync::{Arc, RwLock};
mod ui;

use core::core::Core;
use lua_engine::lua_engine::LuaEngine;
use ui::MyApp;

fn main() -> eframe::Result<()> {
    // Create the Rust Core
    let core = Arc::new(RwLock::new(Core::new()));

    // Create the Lua Engine, exposing the core API to Lua
    let lua_engine = Arc::new(RwLock::new(LuaEngine::new(core)));

    // Run the UI
    let options = eframe::NativeOptions::default();
    let app = MyApp::new(lua_engine.clone());
    if let Err(err) = lua_engine
        .write()
        .unwrap()
        .lua
        .load("require('init')")
        .exec()
    {
        eprintln!("Unable to load init.lua due to lua error: {}", err);
    }
    eframe::run_native(
        "Space Business 5",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
