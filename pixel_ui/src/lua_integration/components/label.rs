use crate::ui::{UiComponent, UiEvent};
use macroquad::prelude::*;
use std::any::Any;

pub struct LabelComponent {
    rect: Rect,
    text: String,
    font_size: f32,
    color: Color,
}

impl LabelComponent {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            text: "Label".to_string(),
            font_size: 20.0,
            color: WHITE,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl UiComponent for LabelComponent {
    fn draw(&self) {
        draw_text(
            &self.text,
            self.rect.x,
            self.rect.y + self.font_size, // Adjust for baseline
            self.font_size,
            self.color,
        );
    }

    fn update(&mut self, _dt: f32) {
        // Labels don't need update logic
    }

    fn handle_event(&mut self, _event: &UiEvent) -> bool {
        // Labels don't handle events
        false
    }

    fn get_rect(&self) -> Rect {
        self.rect
    }

    fn set_position(&mut self, position: Vec2) {
        self.rect.x = position.x;
        self.rect.y = position.y;
    }

    fn set_size(&mut self, size: Vec2) {
        self.rect.w = size.x;
        self.rect.h = size.y;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
