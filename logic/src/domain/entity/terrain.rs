use crate::domain::value_object::location::Location;
use crate::NumericId;
use utils_derive::NumericId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NumericId)]
pub struct TerrainId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NumericId)]
pub struct TerrainTypeId(pub u32);
#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    pub(crate) id: TerrainId,
    pub(crate) terrain_type: TerrainTypeId,
    pub(crate) location: Location,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainType {
    pub(crate) id: TerrainTypeId,
    pub(crate) movement_cost: f32,
}
