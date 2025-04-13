use crate::domain::entity::terrain::{Terrain, TerrainId, TerrainType, TerrainTypeId};
use crate::domain::event::DomainEvent;
use std::sync::mpsc::Sender;
use utils::repo::Repository;

pub struct TerrainService<
    R: Repository<TerrainId, Terrain>,
    S: Repository<TerrainTypeId, TerrainType>,
> {
    terrain_repository: R,
    terrain_type_repository: S,
    event_sender: Sender<DomainEvent>,
}

impl<R: Repository<TerrainId, Terrain>, S: Repository<TerrainTypeId, TerrainType>>
    TerrainService<R, S>
{
    pub fn new(
        terrain_repository: R,
        terrain_type_repository: S,
        event_sender: Sender<DomainEvent>,
    ) -> Self {
        TerrainService {
            terrain_repository,
            terrain_type_repository,
            event_sender,
        }
    }

    pub fn get_terrain(&self, id: TerrainId) -> Result<Terrain, R::Error> {
        self.terrain_repository.get(id)
    }

    pub fn get_terrain_type(&self, id: TerrainTypeId) -> Result<TerrainType, S::Error> {
        self.terrain_type_repository.get(id)
    }
}
