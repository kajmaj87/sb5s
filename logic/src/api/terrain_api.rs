use crate::domain::entity::terrain::{TerrainId, TerrainTypeId};
use crate::domain::value_object::location::Location;
use crate::TerrainApi;

impl TerrainApi {
    /// register a new terrain type
    pub fn register_terrain_type(&self, movement_cost: f32) -> Result<TerrainTypeId, String> {
        self.service
            .lock()
            .register_terrain_type(movement_cost)
            .map_err(|e| format!("Failed to register terrain type: {:?}", e))
    }

    pub fn set_terrain(
        &self,
        terrain_type_id: TerrainTypeId,
        location: Location,
    ) -> Result<TerrainId, String> {
        self.service
            .lock()
            .set_terrain(terrain_type_id, location)
            .map_err(|e| format!("Failed to set terrain: {:?}", e))
    }
}
