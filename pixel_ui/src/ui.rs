use macroquad::prelude::*;

// Reexport components structure - we'll add this file later
pub mod components;

#[derive(Debug, Clone)]
pub enum UiEvent {
    Click { position: Vec2 },
    Hover { position: Vec2 },
    KeyPress { key: KeyCode },
    // Other events as needed
}

// Re-export UiComponent from components
pub use components::UiComponent;
