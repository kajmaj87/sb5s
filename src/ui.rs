use crate::lua_engine::LuaEngine;
use egui_plot::{Line, Plot, PlotPoints};
use mlua::prelude::{LuaFunction, LuaResult};
use std::sync::{Arc, RwLock};

enum UIComponent {
    Button { label: String, handler: LuaFunction },
    Plot { label: String, handler: LuaFunction },
}
pub struct MyApp {
    lua_engine: Arc<LuaEngine>,
    script_input: String,
    components: Arc<RwLock<Vec<UIComponent>>>,
}

impl MyApp {
    pub fn new(lua_engine: Arc<LuaEngine>) -> Self {
        let lua = &lua_engine.lua;

        // Register UI components (buttons, labels, etc.) in Lua
        let globals = lua.globals();
        let components: Arc<RwLock<Vec<UIComponent>>> = Arc::new(RwLock::new(Vec::new()));
        // Register add_button in Lua
        let components_clone = Arc::clone(&components);
        let add_button = lua
            .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                let mut buttons = components_clone.write().unwrap();
                println!("Button added: {}", label);
                buttons.push(UIComponent::Button { label, handler });
                Ok(())
            })
            .unwrap();
        globals.set("button", add_button).unwrap();
        // Register add_plot in Lua
        let components_clone = Arc::clone(&components);
        let add_plot = lua
            .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                let mut plots = components_clone.write().unwrap();
                println!("Plot added: {}", label);
                plots.push(UIComponent::Plot { label, handler });
                Ok(())
            })
            .unwrap();
        globals.set("plot", add_plot).unwrap();
        let components_clone = Arc::clone(&components);
        let reset_components = lua
            .create_function(move |_, ()| {
                let mut components = components_clone.write().unwrap();
                components.clear();
                Ok(())
            })
            .unwrap();
        globals.set("reset", reset_components).unwrap();
        Self {
            lua_engine,
            script_input: String::new(),
            components,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            {
                let components = self.components.read().unwrap();
                for component in components.iter() {
                    match component {
                        UIComponent::Button { label, handler } => {
                            if ui.button(label).clicked() {
                                if let Err(err) = &handler.call::<()>(()) {
                                    eprintln!("Error calling Lua handler: {}", err);
                                }
                            }
                        }
                        UIComponent::Plot { label, handler } => {
                            Plot::new(label)
                                .view_aspect(2.0) // Aspect ratio
                                .show(ui, |plot_ui| {
                                    let result: LuaResult<Vec<f64>> = handler.call(());
                                    if let Ok(data) = result {
                                        plot_ui.line(Line::new(PlotPoints::from_ys_f64(&data)));
                                    } else if let Err(err) = result {
                                        eprintln!("Error generating plot data: {}", err);
                                    }
                                });
                        }
                    };
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
