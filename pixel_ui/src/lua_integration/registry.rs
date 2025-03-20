use macroquad::prelude::*;
use std::collections::HashMap;

use crate::ui::{UiComponent, UiEvent};

pub struct UiComponentRegistry {
    components: HashMap<u32, Box<dyn UiComponent>>,
    next_id: u32,
    root_components: Vec<u32>,
    parent_map: HashMap<u32, u32>, // child_id -> parent_id
}

impl UiComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            next_id: 1, // Start from 1 to avoid 0 (which could be confused with null)
            root_components: Vec::new(),
            parent_map: HashMap::new(),
        }
    }

    pub fn register_component(&mut self, component: Box<dyn UiComponent>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.components.insert(id, component);
        self.root_components.push(id); // Assume it's a root until parented
        id
    }

    pub fn get_component(&self, id: u32) -> Option<&dyn UiComponent> {
        self.components.get(&id).map(|b| b.as_ref())
    }

    pub fn get_component_mut(&mut self, id: u32) -> Option<&mut dyn UiComponent> {
        self.components.get_mut(&id).map(|b| b.as_mut())
    }

    pub fn set_parent(&mut self, child_id: u32, parent_id: u32) -> bool {
        if !self.components.contains_key(&child_id) || !self.components.contains_key(&parent_id) {
            return false;
        }

        // Remove from root if it was there
        if let Some(pos) = self.root_components.iter().position(|&id| id == child_id) {
            self.root_components.swap_remove(pos);
        }

        // Update parent mapping
        self.parent_map.insert(child_id, parent_id);
        true
    }

    pub fn remove_component(&mut self, id: u32) -> bool {
        if !self.components.contains_key(&id) {
            return false;
        }

        // Remove from parent or root
        if let Some(pos) = self.root_components.iter().position(|&i| i == id) {
            self.root_components.swap_remove(pos);
        }
        self.parent_map.remove(&id);

        // Remove all children recursively
        let child_ids: Vec<u32> = self
            .parent_map
            .iter()
            .filter_map(|(&child, &parent)| if parent == id { Some(child) } else { None })
            .collect();

        for child_id in child_ids {
            self.remove_component(child_id);
        }

        // Remove the component itself
        self.components.remove(&id);
        true
    }

    pub fn update_all(&mut self, dt: f32) {
        // Update all components in order
        for id in 0..self.next_id {
            if let Some(component) = self.components.get_mut(&id) {
                component.update(dt);
            }
        }
    }

    pub fn process_event(&mut self, event: &UiEvent) -> bool {
        // Process in reverse order (top to bottom)
        let mut processed = false;

        // Convert to Vec to avoid borrowing issues
        let ids: Vec<u32> = self.components.keys().cloned().collect();

        // Process from highest index (presumably top-most) to lowest
        for id in ids.iter().rev() {
            if let Some(component) = self.components.get_mut(id) {
                if component.handle_event(event) {
                    processed = true;
                    break;
                }
            }
        }

        processed
    }

    pub fn draw_all(&self) {
        // Draw only root components (which will draw their children)
        for &id in &self.root_components {
            if let Some(component) = self.components.get(&id) {
                component.draw();
            }
        }
    }
}
