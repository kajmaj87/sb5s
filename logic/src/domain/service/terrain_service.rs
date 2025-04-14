use crate::domain::entity::terrain::{Terrain, TerrainId, TerrainType, TerrainTypeId};
use crate::domain::event::terrain_events::TerrainEvent;
use crate::domain::event::DomainEvent;
use crate::domain::value_object::location::Location;
use crate::infrastructure::event_store::publish_event;
use std::sync::mpsc::Sender;
use utils::repo::{KeyValueRepository, Repository};

pub struct TerrainService<
    R: Repository<TerrainId, Terrain>,
    S: Repository<TerrainTypeId, TerrainType>,
> {
    terrain_repository: R,
    terrain_type_repository: S,
    terrain_map: KeyValueRepository<Location, TerrainId>,
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
            terrain_map: KeyValueRepository::new(),
            event_sender,
        }
    }

    /// Register a new terrain type
    pub fn register_terrain_type(&mut self, movement_cost: f32) -> Result<TerrainTypeId, S::Error> {
        let terrain_type = self
            .terrain_type_repository
            .create(|id| TerrainType { id, movement_cost })?;

        let event = DomainEvent::Terrain(TerrainEvent::TerrainTypeCreated {
            terrain_type_id: terrain_type.id,
            movement_cost,
        });

        publish_event(&self.event_sender, event);

        Ok(terrain_type.id)
    }

    /// Set a terrain type at a specific location
    pub fn set_terrain(
        &mut self,
        terrain_type_id: TerrainTypeId,
        location: Location,
    ) -> Result<TerrainId, R::Error> {
        if !self.terrain_map.contains(&location) {
            let terrain = self.terrain_repository.create(|id| Terrain {
                id,
                terrain_type: terrain_type_id,
                location: location.clone(),
            })?;
            self.terrain_map.insert(location, terrain.id);

            let event = DomainEvent::Terrain(TerrainEvent::TerrainCreated {
                terrain_id: terrain.id,
                terrain_type_id: terrain.terrain_type,
                location: terrain.location,
            });
            publish_event(&self.event_sender, event);
            Ok(terrain.id)
        } else {
            Ok(self.terrain_map.get(location).unwrap().clone())
        }
    }
}
