use super::registry::UiComponentRegistry;
use lua_engine::lua_engine::LuaEngine;
use lua_engine::LuaResult;
use std::sync::{Arc, Mutex};

/// Register the UI module with the Lua environment
pub fn register_ui_module(
    engine: Arc<LuaEngine>,
    registry: Arc<Mutex<UiComponentRegistry>>,
) -> LuaResult<()> {
    // Register the create function
    {
        let registry = Arc::clone(&registry);

        // engine.register_function(
        //     "create",
        //     "ui",
        //     move |_engine, args: Table| -> Result<u32, String> {
        //         let component_type: String = args
        //             .get("type")
        //             .map_err(|e| format!("Failed to get component type: {}", e))?;
        //
        //         let mut registry = registry.lock().unwrap();
        //         let component: Box<dyn UiComponent> = match component_type.as_str() {
        //             "panel" => Box::new(PanelComponent::new()),
        //             "button" => Box::new(ButtonComponent::new()),
        //             "label" => Box::new(LabelComponent::new()),
        //             _ => {
        //                 return Err(format!("Unknown component type: {}", component_type));
        //             }
        //         };
        //
        //         let id = registry.register_component(component);
        //         Ok(id)
        //     },
        // )?;
    }

    // Register set_props function
    {
        let registry = Arc::clone(&registry);

        // engine.register_function(
        //     "set_props",
        //     "ui",
        //     move |engine, (id, props): (u32, Table)| -> Result<(), String> {
        //         let mut registry = registry.lock().unwrap();
        //         let component = registry
        //             .get_component_mut(id)
        //             .ok_or_else(|| format!("Component with id {} not found", id))?;
        //
        //         // Set common properties
        //         if let Ok(x) = props.get::<Option<f32>>("x") {
        //             if let Some(x_val) = x {
        //                 let rect = component.get_rect();
        //                 component.set_position(Vec2::new(x_val, rect.y));
        //             }
        //         }
        //
        //         if let Ok(y) = props.get::<Option<f32>>("y") {
        //             if let Some(y_val) = y {
        //                 let rect = component.get_rect();
        //                 component.set_position(Vec2::new(rect.x, y_val));
        //             }
        //         }
        //
        //         if let Ok(width) = props.get::<Option<f32>>("width") {
        //             if let Some(w) = width {
        //                 let rect = component.get_rect();
        //                 component.set_size(Vec2::new(w, rect.h));
        //             }
        //         }
        //
        //         if let Ok(height) = props.get::<Option<f32>>("height") {
        //             if let Some(h) = height {
        //                 let rect = component.get_rect();
        //                 component.set_size(Vec2::new(rect.w, h));
        //             }
        //         }
        //
        //         // Handle parent relationship
        //         if let Ok(parent_id) = props.get::<Option<u32>>("parent") {
        //             if let Some(pid) = parent_id {
        //                 registry.set_parent(id, pid);
        //             }
        //         }
        //
        //         // Handle component-specific properties
        //         if let Some(button) = component.as_any_mut().downcast_mut::<ButtonComponent>() {
        //             if let Ok(text) = props.get::<Option<String>>("text") {
        //                 if let Some(text_val) = text {
        //                     button.set_text(text_val);
        //                 }
        //             }
        //
        //             // Handle on_click callback
        //             if let Ok(Some(callback)) = props.get::<Option<Function>>("on_click") {
        //                 // Register the callback to get an ID
        //                 let callback_id = engine.register_callback(callback);
        //
        //                 // Set the callback using only Send-compatible types
        //                 button.set_callback(move || {
        //                     // Execute Lua callback using the ID instead of the Function directly
        //                     if let Err(e) = engine.execute_callback(callback_id, ()) {
        //                         eprintln!("Error in button callback: {}", e);
        //                     }
        //                 });
        //             }
        //         }
        //         // Rest of component-specific handling remains the same
        //         else if let Some(label) = component.as_any_mut().downcast_mut::<LabelComponent>()
        //         {
        //             // Label handling remains the same
        //             if let Ok(text) = props.get::<Option<String>>("text") {
        //                 if let Some(text_val) = text {
        //                     label.set_text(text_val);
        //                 }
        //             }
        //             // Rest of label handling...
        //         } else if let Some(panel) = component.as_any_mut().downcast_mut::<PanelComponent>()
        //         {
        //             // Panel handling remains the same
        //             if let Ok(color) = props.get::<Option<Table>>("color") {
        //                 if let Some(color_table) = color {
        //                     let r: f32 = color_table
        //                         .get(1)
        //                         .map_err(|e| format!("Failed to get color r: {}", e))?;
        //                     let g: f32 = color_table
        //                         .get(2)
        //                         .map_err(|e| format!("Failed to get color g: {}", e))?;
        //                     let b: f32 = color_table
        //                         .get(3)
        //                         .map_err(|e| format!("Failed to get color b: {}", e))?;
        //                     let a: f32 = color_table.get(4).unwrap_or(1.0);
        //
        //                     panel.set_color(Color::new(r, g, b, a));
        //                 }
        //             }
        //             // Rest of panel handling...
        //         }
        //
        //         Ok(())
        //     },
        // )?;
    }

    // Add layout function for declarative UI creation
    {
        let registry = Arc::clone(&registry);

        // engine.register_function(
        //     "layout",
        //     "ui",
        //     move |_engine, layout: Table| -> Result<Vec<u32>, String> {
        //         let mut created_ids = Vec::new();
        //
        //         // Create the component
        //         let component_type: String = layout
        //             .get("type")
        //             .map_err(|e| format!("Failed to get component type: {}", e))?;
        //
        //         let mut registry = registry.lock().unwrap();
        //         let component: Box<dyn UiComponent> = match component_type.as_str() {
        //             "panel" => Box::new(PanelComponent::new()),
        //             "button" => Box::new(ButtonComponent::new()),
        //             "label" => Box::new(LabelComponent::new()),
        //             _ => {
        //                 return Err(format!("Unknown component type: {}", component_type));
        //             }
        //         };
        //
        //         let id = registry.register_component(component);
        //         created_ids.push(id);
        //
        //         // Temporarily drop lock to avoid deadlock with callback closures
        //         drop(registry);
        //
        //         // Use set_props to set properties - this needs to be handled separately
        //         // as we can't easily access the function from here
        //
        //         // Process children if any
        //         if let Ok(children) = layout.get::<Option<Table>>("children") {
        //             if let Some(children_table) = children {
        //                 for pair in children_table.pairs::<Value, Table>() {
        //                     let (_, child_layout) =
        //                         pair.map_err(|e| format!("Failed to get child layout: {}", e))?;
        //
        //                     // This would need special handling
        //                     // Using mock data for now
        //                     let child_ids = vec![0]; // Placeholder
        //
        //                     for &child_id in &child_ids {
        //                         registry.set_parent(child_id, id);
        //                     }
        //
        //                     created_ids.extend(child_ids);
        //                 }
        //             }
        //         }
        //
        //         Ok(created_ids)
        //     },
        // )?;
    }

    // Add remove function
    {
        let registry = Arc::clone(&registry);

        // engine.register_function(
        //     "remove",
        //     "ui",
        //     move |_engine, id: u32| -> Result<bool, String> {
        //         let mut registry = registry.lock().unwrap();
        //         let removed = registry.remove_component(id);
        //         Ok(removed)
        //     },
        // )?;
    }

    Ok(())
}
