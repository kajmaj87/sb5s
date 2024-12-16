use crate::lua_engine::LuaEngine;
use eframe::egui;
use mlua::prelude::*;
use std::sync::{Arc, RwLock};
pub struct UIButton {
    pub label: String,
    pub handler: LuaFunction, // Optional click handler
}
pub struct MyApp {
    lua_engine: Arc<LuaEngine>,
    script_input: String,
    buttons: Arc<RwLock<Vec<UIButton>>>,
}

impl MyApp {
    pub fn new(lua_engine: Arc<LuaEngine>) -> Self {
        let lua = &lua_engine.lua;

        // Register UI components (buttons, labels, etc.) in Lua
        let globals = lua.globals();
        let buttons = Arc::new(RwLock::new(Vec::new()));

        // Register add_label in Lua
        let add_label = lua
            .create_function(move |_, label: String| {
                println!("Label added: {}", label);
                Ok(())
            })
            .unwrap();
        globals.set("add_label", add_label).unwrap();
        // Register add_button in Lua
        let buttons_clone = Arc::clone(&buttons);
        let add_button = lua
            .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                let mut buttons = buttons_clone.write().unwrap();
                println!("Button added: {}", label);
                buttons.push(UIButton { label, handler });
                Ok(())
            })
            .unwrap();
        globals.set("add_button", add_button).unwrap();

        Self {
            lua_engine,
            script_input: String::new(),
            buttons,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Render all buttons
            {
                let buttons = self.buttons.read().unwrap();
                for button in buttons.iter() {
                    if ui.button(&button.label).clicked() {
                        if let Err(err) = &button.handler.call::<()>(()) {
                            eprintln!("Error calling Lua handler: {}", err);
                        }
                    }
                }
            }
            // Multi-line input for Lua script
            ui.label("Lua Script Input:");
            let text_edit_response = ui.add(
                egui::TextEdit::multiline(&mut self.script_input)
                    .hint_text("Write Lua code here...")
                    .desired_rows(10)
                    .code_editor()
                    .lock_focus(true), // Keep focus on this area
            );

            // Check for Ctrl + Enter to run the Lua script
            if text_edit_response.has_focus()
                && ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)
            {
                self.lua_engine.run_script(&self.script_input);
            }
            if ui.button("Run").clicked() {
                self.lua_engine.run_script(&self.script_input);
            }
        });
    }
}
