use crate::lua_engine::LuaEngine;
use egui_plot::{Line, Plot, PlotPoints};
use mlua::prelude::{LuaFunction, LuaResult};
use std::sync::{Arc, RwLock};

struct UIButton {
    label: String,
    handler: LuaFunction,
}

struct UIPlot {
    label: String,
    data: Vec<f64>,
    handler: LuaFunction,
}
pub struct MyApp {
    lua_engine: Arc<LuaEngine>,
    script_input: String,
    buttons: Arc<RwLock<Vec<UIButton>>>,
    plots: Arc<RwLock<Vec<UIPlot>>>,
}

impl MyApp {
    pub fn new(lua_engine: Arc<LuaEngine>) -> Self {
        let lua = &lua_engine.lua;

        // Register UI components (buttons, labels, etc.) in Lua
        let globals = lua.globals();
        let buttons = Arc::new(RwLock::new(Vec::new()));
        let plots = Arc::new(RwLock::new(Vec::new()));
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
        // Register add_plot in Lua
        let plots_clone = Arc::clone(&plots);
        let add_plot = lua
            .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                let mut plots = plots_clone.write().unwrap();
                println!("Plot added: {}", label);
                let result: LuaResult<Vec<f64>> = handler.call(());
                if let Ok(data) = result {
                    plots.push(UIPlot {
                        label,
                        data,
                        handler,
                    });
                } else if let Err(err) = result {
                    eprintln!("Error generating plot data: {}", err);
                }
                Ok(())
            })
            .unwrap();
        globals.set("add_plot", add_plot).unwrap();
        Self {
            lua_engine,
            script_input: String::new(),
            buttons,
            plots,
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
            // Render all plots
            {
                let plots = self.plots.read().unwrap();
                for plot in plots.iter() {
                    Plot::new(&plot.label)
                        .view_aspect(2.0) // Aspect ratio
                        .show(ui, |plot_ui| {
                            let result: LuaResult<Vec<f64>> = plot.handler.call(());
                            if let Ok(data) = result {
                                plot_ui.line(Line::new(PlotPoints::from_ys_f64(&data)));
                            } else if let Err(err) = result {
                                eprintln!("Error generating plot data: {}", err);
                            }
                        });
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
