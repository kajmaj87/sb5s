use crate::config::DRAG_THRESHOLD;
// File: input_system.rs
use crate::TilePosition;
use std::collections::HashMap;

use lua_engine::LuaFunction;
use macroquad::prelude::*;
use std::sync::Arc;

// Enhanced InputEvent with position data
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    KeyDown(Vec<KeyCode>),
    KeyUp(Vec<KeyCode>),
    MouseDown(MouseButton, Vec2),
    MouseUp(MouseButton, Vec2),
    MouseDrag(MouseButton, Vec2, Vec2), // Button, delta, position
    MouseMove(Vec2),
    MouseWheel(f32),
}

// InputEventType is used as a key for event handler registration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputEventType {
    KeyDown(Vec<KeyCode>),
    KeyUp(Vec<KeyCode>),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseDrag(MouseButton),
    MouseMove,
    MouseWheel,
}

// Part 1: Input State Tracking & Event Generation
pub struct InputManager {
    mouse_position: Vec2,
    prev_mouse_position: Vec2,
    is_dragging: bool,
    drag_start_position: Vec2,
    mouse_moved_during_click: bool,
    zoom_delta: Option<f32>,
    keymap: HashMap<&'static str, KeyCode>,
    // Add tracking of modifier keys
    modifier_state: HashMap<KeyCode, bool>,
    // Events collected during update
    collected_events: Vec<InputEvent>,
}

impl InputManager {
    pub fn new() -> Self {
        let initial_pos = Vec2::new(mouse_position().0, mouse_position().1);

        // Initialize modifier state
        let mut modifier_state = HashMap::new();
        modifier_state.insert(KeyCode::LeftShift, false);
        modifier_state.insert(KeyCode::RightShift, false);
        modifier_state.insert(KeyCode::LeftControl, false);
        modifier_state.insert(KeyCode::RightControl, false);
        modifier_state.insert(KeyCode::LeftAlt, false);
        modifier_state.insert(KeyCode::RightAlt, false);

        Self {
            mouse_position: initial_pos,
            prev_mouse_position: initial_pos,
            is_dragging: false,
            drag_start_position: initial_pos,
            mouse_moved_during_click: false,
            zoom_delta: None,
            keymap: get_keycode_map(),
            modifier_state,
            collected_events: Vec::new(),
        }
    }

    // Track modifiers independently
    fn update_modifier_state(&mut self) {
        // List of modifier keys to track
        let modifiers = [
            KeyCode::LeftShift,
            KeyCode::RightShift,
            KeyCode::LeftControl,
            KeyCode::RightControl,
            KeyCode::LeftAlt,
            KeyCode::RightAlt,
        ];

        for &key in &modifiers {
            // Check if this modifier was just pressed
            if is_key_pressed(key) {
                self.modifier_state.insert(key, true);
                self.collected_events.push(InputEvent::KeyDown(vec![key]));
            }

            // Check if this modifier was just released
            if is_key_released(key) {
                self.modifier_state.insert(key, false);
                self.collected_events.push(InputEvent::KeyUp(vec![key]));
            }
        }
    }

    // Check which modifiers are currently active
    fn get_active_modifiers(&self) -> Vec<KeyCode> {
        let mut active = Vec::new();

        // Add shift if either left or right shift is down
        if self.modifier_state.get(&KeyCode::LeftShift) == Some(&true)
            || self.modifier_state.get(&KeyCode::RightShift) == Some(&true)
        {
            active.push(KeyCode::LeftShift);
        }

        // Add ctrl if either left or right ctrl is down
        if self.modifier_state.get(&KeyCode::LeftControl) == Some(&true)
            || self.modifier_state.get(&KeyCode::RightControl) == Some(&true)
        {
            active.push(KeyCode::LeftControl);
        }

        // Add alt if either left or right alt is down
        if self.modifier_state.get(&KeyCode::LeftAlt) == Some(&true)
            || self.modifier_state.get(&KeyCode::RightAlt) == Some(&true)
        {
            active.push(KeyCode::LeftAlt);
        }

        active
    }

    pub fn update(&mut self) {
        // Clear previous events
        self.collected_events.clear();

        // Update state
        self.prev_mouse_position = self.mouse_position;
        self.mouse_position = Vec2::new(mouse_position().0, mouse_position().1);

        // Update modifier state FIRST so other handlers have access to it
        self.update_modifier_state();

        // Track mouse wheel
        let wheel_y = mouse_wheel().1;
        self.zoom_delta = if wheel_y != 0.0 {
            self.collected_events.push(InputEvent::MouseWheel(wheel_y));
            Some(wheel_y)
        } else {
            None
        };

        // Track mouse movement
        if self.mouse_position != self.prev_mouse_position {
            self.collected_events
                .push(InputEvent::MouseMove(self.mouse_position));
        }

        // Handle mouse button press/release
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            if is_mouse_button_pressed(button) {
                if button == MouseButton::Left {
                    self.drag_start_position = self.mouse_position;
                    self.is_dragging = false;
                    self.mouse_moved_during_click = false;
                }
                self.collected_events
                    .push(InputEvent::MouseDown(button, self.mouse_position));
            }

            if is_mouse_button_released(button) {
                if button == MouseButton::Left && !self.mouse_moved_during_click {
                    // Special case for clean clicks (not after drag)
                }
                self.collected_events
                    .push(InputEvent::MouseUp(button, self.mouse_position));
            }
        }

        // Check for drag while mouse is down
        if is_mouse_button_down(MouseButton::Left) {
            if !self.is_dragging {
                let delta = self.mouse_position - self.drag_start_position;
                let distance = delta.length();

                if distance > DRAG_THRESHOLD {
                    self.is_dragging = true;
                    self.mouse_moved_during_click = true;
                }
            }

            if self.is_dragging {
                let delta = self.mouse_position - self.prev_mouse_position;
                if delta != Vec2::ZERO {
                    self.collected_events.push(InputEvent::MouseDrag(
                        MouseButton::Left,
                        delta,
                        self.mouse_position,
                    ));
                }
            }
        } else {
            // If left button is not down, we're not dragging
            self.is_dragging = false;
        }

        // Check for regular (non-modifier) keys
        for key in self.keymap.values() {
            // Skip modifier keys - they're handled separately
            if matches!(
                *key,
                KeyCode::LeftShift
                    | KeyCode::RightShift
                    | KeyCode::LeftControl
                    | KeyCode::RightControl
                    | KeyCode::LeftAlt
                    | KeyCode::RightAlt
            ) {
                continue;
            }

            if is_key_pressed(*key) {
                // Get active modifiers and add to key combo
                let mut keys = vec![*key];
                keys.extend(self.get_active_modifiers());

                self.collected_events.push(InputEvent::KeyDown(keys));
            }

            if is_key_released(*key) {
                // For key up events, include active modifiers too
                let mut keys = vec![*key];
                keys.extend(self.get_active_modifiers());

                self.collected_events.push(InputEvent::KeyUp(keys));
            }
        }
    }

    pub fn get_events(&self) -> Vec<InputEvent> {
        self.collected_events.clone()
    }

    // Access to the keymap for the event processor
    pub fn get_keymap(&self) -> &HashMap<&'static str, KeyCode> {
        &self.keymap
    }

    // State query methods remain the same
    pub fn is_direction_pressed(&self) -> bool {
        is_key_down(KeyCode::W)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::S)
            || is_key_down(KeyCode::Down)
            || is_key_down(KeyCode::A)
            || is_key_down(KeyCode::Left)
            || is_key_down(KeyCode::D)
            || is_key_down(KeyCode::Right)
    }

    pub fn is_up_pressed(&self) -> bool {
        is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)
    }

    pub fn is_down_pressed(&self) -> bool {
        is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)
    }

    pub fn is_left_pressed(&self) -> bool {
        is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)
    }

    pub fn is_right_pressed(&self) -> bool {
        is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)
    }

    pub fn is_left_mouse_click(&self) -> bool {
        is_mouse_button_released(MouseButton::Left) && !self.mouse_moved_during_click
    }

    pub fn should_place_tile(&self, selected_pos: Option<&TilePosition>) -> bool {
        is_mouse_button_down(MouseButton::Right) && selected_pos.is_some()
    }

    pub fn get_drag_delta(&self) -> Option<Vec2> {
        if self.is_dragging {
            Some(self.mouse_position - self.prev_mouse_position)
        } else {
            None
        }
    }

    pub fn get_zoom_delta(&self) -> Option<f32> {
        self.zoom_delta
    }

    pub fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }
}

// Part 2: Event Processing & Handler Management
pub struct InputEventProcessor {
    event_handlers: HashMap<InputEventType, Arc<LuaFunction>>,
}
impl InputEventProcessor {
    pub fn new() -> Self {
        Self {
            event_handlers: HashMap::new(),
        }
    }

    // Process events from InputManager
    pub fn process_events(&self, events: Vec<InputEvent>) {
        for event in events {
            match &event {
                InputEvent::KeyDown(keys) => {
                    let event_type = InputEventType::KeyDown(keys.clone());
                    if let Some(handler) = self.event_handlers.get(&event_type) {
                        if let Err(e) = handler.call::<()>(()) {
                            eprintln!("Error calling key handler: {:?}", e);
                        }
                    }
                }
                InputEvent::KeyUp(keys) => {
                    let event_type = InputEventType::KeyUp(keys.clone());
                    if let Some(handler) = self.event_handlers.get(&event_type) {
                        if let Err(e) = handler.call::<()>(()) {
                            eprintln!("Error calling key up handler: {:?}", e);
                        }
                    }
                }
                InputEvent::MouseDown(button, position) => {
                    let event_type = InputEventType::MouseDown(*button);
                    if let Some(handler) = self.event_handlers.get(&event_type) {
                        if let Err(e) = handler.call::<()>((position.x, position.y)) {
                            eprintln!("Error calling mouse down handler: {:?}", e);
                        }
                    }
                }
                InputEvent::MouseUp(button, position) => {
                    let event_type = InputEventType::MouseUp(*button);
                    if let Some(handler) = self.event_handlers.get(&event_type) {
                        if let Err(e) = handler.call::<()>((position.x, position.y)) {
                            eprintln!("Error calling mouse up handler: {:?}", e);
                        }
                    }
                }
                InputEvent::MouseDrag(button, delta, position) => {
                    let event_type = InputEventType::MouseDrag(*button);
                    if let Some(handler) = self.event_handlers.get(&event_type) {
                        if let Err(e) =
                            handler.call::<()>((delta.x, delta.y, position.x, position.y))
                        {
                            eprintln!("Error calling mouse drag handler: {:?}", e);
                        }
                    }
                }
                InputEvent::MouseMove(position) => {
                    if let Some(handler) = self.event_handlers.get(&InputEventType::MouseMove) {
                        if let Err(e) = handler.call::<()>((position.x, position.y)) {
                            eprintln!("Error calling mouse move handler: {:?}", e);
                        }
                    }
                }
                InputEvent::MouseWheel(delta) => {
                    if let Some(handler) = self.event_handlers.get(&InputEventType::MouseWheel) {
                        if let Err(e) = handler.call::<()>(*delta) {
                            eprintln!("Error calling mouse wheel handler: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    // Helper method to parse string into InputEventType
    fn parse_event(
        &self,
        event_id: &str,
        keymap: &HashMap<&'static str, KeyCode>,
    ) -> Option<InputEventType> {
        // Handle drag events
        if event_id.ends_with("_DRAG") {
            let button_id = event_id.trim_end_matches("_DRAG");
            match button_id {
                "LMB" => Some(InputEventType::MouseDrag(MouseButton::Left)),
                "RMB" => Some(InputEventType::MouseDrag(MouseButton::Right)),
                "MMB" => Some(InputEventType::MouseDrag(MouseButton::Middle)),
                _ => None,
            }
        }
        // Handle key up events
        else if event_id.ends_with("_UP") {
            let base_id = event_id.trim_end_matches("_UP");

            // Check if it's a mouse button
            match base_id {
                "LMB" => Some(InputEventType::MouseUp(MouseButton::Left)),
                "RMB" => Some(InputEventType::MouseUp(MouseButton::Right)),
                "MMB" => Some(InputEventType::MouseUp(MouseButton::Middle)),
                _ => {
                    // Handle key combinations with modifiers
                    if base_id.contains("+") {
                        let mut keycodes = base_id
                            .split('+')
                            .filter_map(|key| {
                                keymap.get(key.trim().to_uppercase().as_str()).cloned()
                            })
                            .collect::<Vec<KeyCode>>();

                        // Normalize order of keys (main key first, modifiers after)
                        self.normalize_key_order(&mut keycodes);

                        if !keycodes.is_empty() {
                            Some(InputEventType::KeyUp(keycodes))
                        } else {
                            None
                        }
                    }
                    // Single key
                    else if let Some(&keycode) = keymap.get(base_id.to_uppercase().as_str()) {
                        Some(InputEventType::KeyUp(vec![keycode]))
                    } else {
                        None
                    }
                }
            }
        }
        // Handle mouse specific events
        else if event_id == "MOUSE_MOVE" {
            Some(InputEventType::MouseMove)
        } else if event_id == "MOUSE_WHEEL" {
            Some(InputEventType::MouseWheel)
        }
        // Handle mouse buttons and key combinations
        else {
            // Check for mouse buttons
            match event_id {
                "LMB" => Some(InputEventType::MouseDown(MouseButton::Left)),
                "RMB" => Some(InputEventType::MouseDown(MouseButton::Right)),
                "MMB" => Some(InputEventType::MouseDown(MouseButton::Middle)),
                _ => {
                    // Handle key combinations with modifiers
                    if event_id.contains("+") {
                        let mut keycodes = event_id
                            .split('+')
                            .filter_map(|key| {
                                keymap.get(key.trim().to_uppercase().as_str()).cloned()
                            })
                            .collect::<Vec<KeyCode>>();

                        // Normalize order of keys (main key first, modifiers after)
                        self.normalize_key_order(&mut keycodes);

                        if !keycodes.is_empty() {
                            Some(InputEventType::KeyDown(keycodes))
                        } else {
                            None
                        }
                    }
                    // Single key
                    else if let Some(&keycode) = keymap.get(event_id.to_uppercase().as_str()) {
                        Some(InputEventType::KeyDown(vec![keycode]))
                    } else {
                        None
                    }
                }
            }
        }
    }

    // Helper to normalize the order of keys in key combinations
    fn normalize_key_order(&self, keys: &mut [KeyCode]) {
        // Define a function to check if a key is a modifier
        let is_modifier = |key: &KeyCode| {
            matches!(
                key,
                KeyCode::LeftShift
                    | KeyCode::RightShift
                    | KeyCode::LeftControl
                    | KeyCode::RightControl
                    | KeyCode::LeftAlt
                    | KeyCode::RightAlt
            )
        };

        // Sort keys: non-modifiers first, then modifiers
        keys.sort_by(|a, b| {
            let a_is_mod = is_modifier(a);
            let b_is_mod = is_modifier(b);

            if a_is_mod && !b_is_mod {
                std::cmp::Ordering::Greater
            } else if !a_is_mod && b_is_mod {
                std::cmp::Ordering::Less
            } else {
                // If both are modifiers or both are not, sort by their numeric value
                (*a as u32).cmp(&(*b as u32))
            }
        });
    }

    // Registration methods
    pub fn register_shortcut(
        &mut self,
        key_combo: &str,
        handler: LuaFunction,
        keymap: &HashMap<&'static str, KeyCode>,
    ) -> Result<(), String> {
        // Convert string to InputEventType
        let event_type = match self.parse_event(key_combo, keymap) {
            Some(event) => event,
            None => return Err(format!("Invalid key combination: {}", key_combo)),
        };

        // Store the handler
        self.event_handlers.insert(event_type, Arc::new(handler));
        Ok(())
    }

    // Register a mouse button handler
    pub fn register_mouse(
        &mut self,
        button_id: &str,
        handler: LuaFunction,
        keymap: &HashMap<&'static str, KeyCode>,
    ) -> Result<(), String> {
        // Convert string to InputEventType
        let event_type = match self.parse_event(button_id, keymap) {
            Some(event) => event,
            None => return Err(format!("Invalid mouse button: {}", button_id)),
        };

        // Store the handler
        self.event_handlers.insert(event_type, Arc::new(handler));
        Ok(())
    }

    // Register a mouse drag handler
    pub fn register_drag(
        &mut self,
        button_id: &str,
        handler: LuaFunction,
        keymap: &HashMap<&'static str, KeyCode>,
    ) -> Result<(), String> {
        // Convert string to drag event ID
        let drag_id = format!("{}_DRAG", button_id);

        // Parse into event type
        let event_type = match self.parse_event(&drag_id, keymap) {
            Some(event) => event,
            None => return Err(format!("Invalid drag button: {}", button_id)),
        };

        // Store the handler
        self.event_handlers
            .entry(event_type)
            .or_insert(Arc::new(handler));
        Ok(())
    }

    // Register a mouse move handler
    pub fn register_mouse_move(&mut self, handler: LuaFunction) -> Result<(), String> {
        self.event_handlers
            .entry(InputEventType::MouseMove)
            .or_insert(Arc::new(handler));
        Ok(())
    }

    // Register a mouse wheel handler
    pub fn register_mouse_wheel(&mut self, handler: LuaFunction) -> Result<(), String> {
        self.event_handlers
            .insert(InputEventType::MouseWheel, Arc::new(handler));
        Ok(())
    }

    // Unregister by string ID
    pub fn unregister(&mut self, event_id: &str, keymap: &HashMap<&'static str, KeyCode>) -> bool {
        if let Some(event_type) = self.parse_event(event_id, keymap) {
            self.event_handlers.remove(&event_type).is_some()
        } else {
            false
        }
    }
}
pub fn get_keycode_map() -> HashMap<&'static str, KeyCode> {
    // Same as before
    let mut map = HashMap::new();
    for key in [
        ("SPACE", KeyCode::Space),
        ("APOSTROPHE", KeyCode::Apostrophe),
        ("COMMA", KeyCode::Comma),
        ("MINUS", KeyCode::Minus),
        ("PERIOD", KeyCode::Period),
        ("SLASH", KeyCode::Slash),
        ("F1", KeyCode::F1),
        ("F2", KeyCode::F2),
        ("F3", KeyCode::F3),
        ("F4", KeyCode::F4),
        ("F5", KeyCode::F5),
        ("F6", KeyCode::F6),
        ("F7", KeyCode::F7),
        ("F8", KeyCode::F8),
        ("F9", KeyCode::F9),
        ("F10", KeyCode::F10),
        ("F11", KeyCode::F11),
        ("F12", KeyCode::F12),
        ("A", KeyCode::A),
        ("B", KeyCode::B),
        ("C", KeyCode::C),
        ("D", KeyCode::D),
        ("E", KeyCode::E),
        ("F", KeyCode::F),
        ("G", KeyCode::G),
        ("H", KeyCode::H),
        ("I", KeyCode::I),
        ("J", KeyCode::J),
        ("K", KeyCode::K),
        ("L", KeyCode::L),
        ("M", KeyCode::M),
        ("N", KeyCode::N),
        ("O", KeyCode::O),
        ("P", KeyCode::P),
        ("Q", KeyCode::Q),
        ("R", KeyCode::R),
        ("S", KeyCode::S),
        ("T", KeyCode::T),
        ("U", KeyCode::U),
        ("V", KeyCode::V),
        ("W", KeyCode::W),
        ("X", KeyCode::X),
        ("Y", KeyCode::Y),
        ("Z", KeyCode::Z),
        // Add more keys as needed, like function keys, control keys, etc.
        ("ESCAPE", KeyCode::Escape),
        ("ENTER", KeyCode::Enter),
        ("TAB", KeyCode::Tab),
        ("BACKSPACE", KeyCode::Backspace),
        ("INSERT", KeyCode::Insert),
        ("DELETE", KeyCode::Delete),
        ("RIGHT", KeyCode::Right),
        ("LEFT", KeyCode::Left),
        ("DOWN", KeyCode::Down),
        ("UP", KeyCode::Up),
        ("PAGE_UP", KeyCode::PageUp),
        ("PAGE_DOWN", KeyCode::PageDown),
        ("HOME", KeyCode::Home),
        ("END", KeyCode::End),
        ("SHIFT", KeyCode::LeftShift),
        ("CTRL", KeyCode::LeftControl),
        ("ALT", KeyCode::LeftAlt),
    ] {
        map.insert(key.0, key.1);
    }
    map
}
