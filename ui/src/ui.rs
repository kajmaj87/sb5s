use egui::Window;
use egui_plot::{Line, Plot, PlotPoints};
use lua_engine::lua_engine::LuaEngine;
use mlua::prelude::LuaFunction;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

enum UIComponent {
    Button {
        label: String,
        handler: LuaFunction,
    },
    TextEdit {
        label: String,
        handler: LuaFunction,
    },
    Label {
        handler: LuaFunction,
    },
    Slider {
        label: String,
        handler: LuaFunction,
    },
    Plot {
        label: String,
        handler: LuaFunction,
    },
    Window {
        label: String,
        children: Vec<UIComponent>,
    },
    LuaConsole {
        script: String,
    },
}
pub struct MyApp {
    lua_engine: Arc<RwLock<LuaEngine>>,
    script_input: String,
    components: Arc<RwLock<Vec<UIComponent>>>,
    new_components: Arc<RwLock<Vec<UIComponent>>>,
}

impl MyApp {
    pub fn new(lua_engine: Arc<RwLock<LuaEngine>>) -> Self {
        let components: Arc<RwLock<Vec<UIComponent>>> = Arc::new(RwLock::new(Vec::new()));
        let old_components = Arc::new(RwLock::new(Vec::new()));
        {
            let lua = &lua_engine.write().unwrap().lua;

            // Register UI components (buttons, labels, etc.) in Lua
            let globals = lua.globals();
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
            // Register add_text_edit in Lua
            let components_clone = Arc::clone(&components);
            let add_text_edit = lua
                .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                    let mut text_edits = components_clone.write().unwrap();
                    println!("TextEdit added: {}", label);
                    text_edits.push(UIComponent::TextEdit { label, handler });
                    Ok(())
                })
                .unwrap();
            globals.set("text_edit", add_text_edit).unwrap();
            // Register add_label in Lua
            let components_clone = Arc::clone(&components);
            let add_label = lua
                .create_function(move |lua_ctx, handler: LuaFunction| {
                    let mut labels = components_clone.write().unwrap();
                    labels.push(UIComponent::Label { handler });
                    Ok(())
                })
                .unwrap();
            globals.set("label", add_label).unwrap();
            let components_clone = Arc::clone(&components);
            let add_slider = lua
                .create_function(move |lua_ctx, (label, handler): (String, LuaFunction)| {
                    let mut text_edits = components_clone.write().unwrap();
                    println!("Slider added: {}", label);
                    text_edits.push(UIComponent::Slider { label, handler });
                    Ok(())
                })
                .unwrap();
            globals.set("slider", add_slider).unwrap();
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
            let add_window = lua
                .create_function(move |_, (label, child_func): (String, LuaFunction)| {
                    let mut children = Vec::new(); // Temporary list to store components inside this window
                                                   // Call the Lua function to define components inside the window
                    {
                        let mut components_lock = components_clone.write().unwrap();
                        std::mem::swap(&mut children, &mut *components_lock);
                    }
                    // Call the child function: this will add components to `components_clone`
                    child_func.call::<()>(())?;
                    // Move the components added by `child_func` back into `children`
                    {
                        let mut components_lock = components_clone.write().unwrap();
                        std::mem::swap(&mut children, &mut *components_lock);
                    }
                    // Add the new window (with its children) to the main list of components
                    let mut components_lock = components_clone.write().unwrap();
                    components_lock.push(UIComponent::Window { label, children });
                    Ok(())
                })
                .unwrap();
            globals.set("window", add_window).unwrap();
            let components_clone = Arc::clone(&components);
            let lua_console = lua
                .create_function(move |lua_ctx, (script): (String)| {
                    let mut components = components_clone.write().unwrap();
                    components.push(UIComponent::LuaConsole { script });
                    Ok(())
                })
                .unwrap();
            globals.set("lua_console", lua_console).unwrap();
            let components_clone = Arc::clone(&old_components);
            let reset_components = lua
                .create_function(move |_, ()| {
                    let mut components = components_clone.write().unwrap();
                    components.clear();
                    Ok(())
                })
                .unwrap();
            globals.set("reset", reset_components).unwrap();
        }
        Self {
            lua_engine,
            script_input: String::new(),
            components: old_components,
            new_components: components,
        }
    }

    fn render_component(
        lua_engine: &RwLockWriteGuard<LuaEngine>,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        component: &mut UIComponent,
    ) {
        match component {
            UIComponent::Button { label, handler } => {
                if ui.button(label.clone()).clicked() {
                    if let Err(err) = handler.call::<()>(()) {
                        eprintln!("Error calling Lua handler: {}", err);
                    }
                }
            }
            UIComponent::TextEdit { label, handler } => {
                let mut value = handler.call::<String>(()).unwrap_or_default();
                let response = ui.text_edit_singleline(&mut value);
                if response.changed() {
                    // Send the new value back to Lua
                    if let Err(err) = handler.call::<String>(value) {
                        eprintln!("Error updating TextEdit value: {}", err);
                    }
                }
            }
            UIComponent::Label { handler } => {
                if let Ok(value) = handler.call::<String>(()) {
                    ui.label(&value);
                } else {
                    eprintln!("Error fetching Label value.");
                }
            }
            UIComponent::Slider { label, handler } => {
                let mut value = handler.call::<f64>(()).unwrap_or_default();
                let response = ui.add(egui::Slider::new(&mut value, 0.0..=100.0));
                if response.changed() {
                    // Send the new value back to Lua
                    if let Err(err) = handler.call::<String>(value) {
                        eprintln!("Error updating Slider value: {}", err);
                    }
                }
            }
            UIComponent::Plot { label, handler } => {
                Plot::new(label).view_aspect(2.0).show(ui, |plot_ui| {
                    if let Ok(data) = handler.call::<Vec<f64>>(()) {
                        plot_ui.line(Line::new(PlotPoints::from_ys_f64(&data)));
                    }
                });
            }
            UIComponent::Window { label, children } => {
                Window::new(label.clone()).show(ctx, |ui| {
                    for child in children {
                        Self::render_component(lua_engine, ctx, ui, child);
                    }
                });
            }
            UIComponent::LuaConsole { script } => {
                // Multi-line input for Lua script
                let text_edit_response = ui.add(
                    egui::TextEdit::multiline(script)
                        .hint_text("Write Lua code here...")
                        .desired_rows(10)
                        .code_editor()
                        .lock_focus(true), // Keep focus on this area
                );
                // Check for Ctrl + Enter to run the Lua script
                if (text_edit_response.has_focus()
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl))
                    || ui.button("Run (Ctrl + Enter)").clicked()
                {
                    lua_engine.run_script(&script);
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Append any new components to the main list of components
        {
            let mut components = self.components.write().unwrap();
            let mut new_components = self.new_components.write().unwrap();
            components.append(&mut new_components);
            new_components.clear()
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut components = self.components.write().unwrap();
            let lua_engine = self.lua_engine.write().unwrap();
            for component in components.iter_mut() {
                Self::render_component(&lua_engine, ctx, ui, component);
            }
        });
    }
}
