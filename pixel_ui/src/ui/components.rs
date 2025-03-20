use super::UiEvent;
use macroquad::prelude::*;
use std::any::Any;

pub trait UiComponent: Any + Send + Sync {
    fn draw(&self);
    fn update(&mut self, dt: f32);
    fn handle_event(&mut self, event: &UiEvent) -> bool;
    fn get_rect(&self) -> Rect;
    fn set_position(&mut self, position: Vec2);
    fn set_size(&mut self, size: Vec2);

    // For downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
