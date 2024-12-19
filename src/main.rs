use std::sync::{Arc, RwLock};
mod core;
mod lua_engine;
mod ui;

use core::Core;
use lua_engine::LuaEngine;
use ui::MyApp;

fn main() -> eframe::Result<()> {
    // Create the Rust Core
    let core = Arc::new(RwLock::new(Core::new()));

    // Create the Lua Engine, exposing the core API to Lua
    let lua_engine = Arc::new(LuaEngine::new(core));

    // Run the UI
    let options = eframe::NativeOptions::default();
    let app = MyApp::new(lua_engine.clone());
    if let Err(err) = lua_engine.lua.load("require('init')").exec() {
        eprintln!("Unable to load init.lua due to lua error: {}", err);
    }
    eframe::run_native(
        "Space Business 5",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
