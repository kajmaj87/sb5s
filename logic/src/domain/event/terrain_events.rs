use crate::domain::entity::terrain::{TerrainId, TerrainTypeId};
use crate::domain::value_object::location::Location;

#[derive(Debug, Clone, PartialEq)]
pub enum TerrainEvent {
    TerrainTypeCreated {
        terrain_type_id: TerrainTypeId,
        movement_cost: f32,
    },
    TerrainCreated {
        terrain_id: TerrainId,
        terrain_type_id: TerrainTypeId,
        location: Location,
    },
}
