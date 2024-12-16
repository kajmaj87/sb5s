use crate::core::Core;
use mlua::prelude::*;
use std::sync::{Arc, RwLock};

pub struct LuaEngine {
    pub lua: Lua,
}

impl LuaEngine {
    pub fn new(core: Arc<RwLock<Core>>) -> Self {
        let lua = Lua::new();
        let core_clone = Arc::clone(&core);

        let globals = lua.globals();

        // Expose core.add_agent to Lua
        let add_agent = lua
            .create_function(move |_, name: String| {
                core_clone.write().unwrap().add_agent(name);
                Ok(())
            })
            .unwrap();
        globals.set("add_agent", add_agent).unwrap();

        // Expose core.move_agent to Lua
        let core_clone = Arc::clone(&core);
        let move_agent = lua
            .create_function(move |_, (name, location): (String, String)| {
                core_clone.read().unwrap().move_agent(&name, &location);
                Ok(())
            })
            .unwrap();
        globals.set("move_agent", move_agent).unwrap();

        // Expose core.get_agents to Lua
        let core_clone = Arc::clone(&core);
        let get_agents = lua
            .create_function(move |_, ()| {
                let agents = core_clone.read().unwrap().get_agents();
                Ok(agents)
            })
            .unwrap();
        globals.set("get_agents", get_agents).unwrap();
        Self { lua }
    }

    pub fn run_script(&self, script: &str) {
        if let Err(err) = self.lua.load(script).exec() {
            eprintln!("Lua Error: {}", err);
        }
    }
}
