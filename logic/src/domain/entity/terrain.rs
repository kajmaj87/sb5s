use crate::domain::value_object::location::Location;
use utils::repo::NumericId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainId(pub u32);
impl NumericId for TerrainId {
    fn value(&self) -> u32 {
        self.0
    }

    fn from_value(value: u32) -> Self {
        TerrainId(value)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainTypeId(pub u32);
impl NumericId for TerrainTypeId {
    fn value(&self) -> u32 {
        self.0
    }

    fn from_value(value: u32) -> Self {
        TerrainTypeId(value)
    }
}

pub struct Terrain {
    id: TerrainId,
    terrain_type: TerrainTypeId,
    location: Location,
}
pub struct TerrainType {
    id: TerrainTypeId,
    name: String,
    movement_cost: f32,
    buildable: bool,
}

pub struct TerrainConnection {
    center: TerrainTypeId,
    north: Option<TerrainTypeId>,
    south: Option<TerrainTypeId>,
    east: Option<TerrainTypeId>,
    west: Option<TerrainTypeId>,
}
