use crate::domain::event::person_event::PersonEvent;
use crate::domain::event::terrain_events::TerrainEvent;

pub mod person_event;
pub mod terrain_events;

#[derive(Debug, Clone, PartialEq)]
pub enum DomainEvent {
    Person(PersonEvent),
    Terrain(TerrainEvent),
    // Other event types can be added here
}
