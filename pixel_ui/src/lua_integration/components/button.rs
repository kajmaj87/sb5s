use crate::ui::{UiComponent, UiEvent};
use macroquad::prelude::*;
use std::any::Any;

pub struct ButtonComponent {
    rect: Rect,
    text: String,
    color: Color,
    hover_color: Color,
    text_color: Color,
    is_hovered: bool,
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ButtonComponent {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 100.0, 30.0),
            text: "Button".to_string(),
            color: Color::new(0.3, 0.3, 0.3, 1.0),
            hover_color: Color::new(0.4, 0.4, 0.4, 1.0),
            text_color: WHITE,
            is_hovered: false,
            callback: None,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_callback<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.callback = Some(Box::new(callback));
    }
}

impl UiComponent for ButtonComponent {
    fn draw(&self) {
        let color = if self.is_hovered {
            self.hover_color
        } else {
            self.color
        };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);

        // Calculate text position
        let text_size = measure_text(&self.text, None, 20, 1.0);
        let text_x = self.rect.x + (self.rect.w - text_size.width) / 2.0;
        let text_y = self.rect.y + (self.rect.h + text_size.height) / 2.0;

        draw_text(&self.text, text_x, text_y, 20.0, self.text_color);
    }

    fn update(&mut self, _dt: f32) {
        // Update hover state
        let mouse_pos = mouse_position();
        self.is_hovered = self.rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1));
    }

    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Click { position } => {
                if self.rect.contains(*position) {
                    if let Some(callback) = &self.callback {
                        callback();
                    }
                    return true;
                }
            }
            UiEvent::Hover { position } => {
                self.is_hovered = self.rect.contains(*position);
                return self.is_hovered;
            }
            _ => {}
        }
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
