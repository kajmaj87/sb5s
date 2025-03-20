use crate::ui::{UiComponent, UiEvent};
use macroquad::prelude::*;
use std::any::Any;

pub struct PanelComponent {
    rect: Rect,
    color: Color,
    border_color: Option<Color>,
    border_width: f32,
}

impl PanelComponent {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 200.0, 150.0),
            color: Color::new(0.2, 0.2, 0.2, 0.8),
            border_color: None,
            border_width: 1.0,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_border(&mut self, color: Color, width: f32) {
        self.border_color = Some(color);
        self.border_width = width;
    }
}

impl UiComponent for PanelComponent {
    fn draw(&self) {
        // Draw main panel
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            self.color,
        );

        // Draw border if set
        if let Some(border_color) = self.border_color {
            draw_rectangle_lines(
                self.rect.x,
                self.rect.y,
                self.rect.w,
                self.rect.h,
                self.border_width,
                border_color,
            );
        }
    }

    fn update(&mut self, _dt: f32) {
        // Panels don't need update logic
    }

    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Click { position } => {
                // Just detect clicks for now, no action
                return self.rect.contains(*position);
            }
            _ => false,
        }
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
