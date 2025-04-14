mod event_api;
mod location_api;
mod person_api;
mod projection_api;
mod terrain_api;

use crate::domain::service::person_service::PersonService;
use crate::domain::service::terrain_service::TerrainService;
use crate::infrastructure::event_store::{create_event_store, EventStore};
use crate::infrastructure::projection::{LocationOccupancyProjection, ProjectionManager};
use parking_lot::Mutex;
use std::sync::Arc;
use utils::repo::VecRepository;

pub use crate::domain::entity::person::Person;
use crate::domain::entity::person::PersonId;
pub use crate::domain::entity::terrain::TerrainTypeId;
use crate::domain::entity::terrain::{Terrain, TerrainId, TerrainType};
use crate::domain::event::DomainEvent;
pub use crate::domain::value_object::location::Location;

/// Main API facade for the logic module
pub struct CoreApi {
    person: PersonApi,
    location: LocationApi,
    event: EventApi,
    projection: ProjectionApi,
    terrain: TerrainApi,
}
/// API for person-related operations
pub struct PersonApi {
    service: Arc<Mutex<PersonService<VecRepository<PersonId, Person>>>>,
}

pub struct TerrainApi {
    service: Arc<
        Mutex<
            TerrainService<
                VecRepository<TerrainId, Terrain>,
                VecRepository<TerrainTypeId, TerrainType>,
            >,
        >,
    >,
}

/// API for location-related queries
pub struct LocationApi {
    projection: Arc<Mutex<LocationOccupancyProjection>>,
}

// Projection trait and manager
pub trait Projection: Send + 'static {
    /** Apply a single event to update the projection state */
    fn apply(&mut self, event: &DomainEvent);

    /** Optional method to initialize the projection before replaying events */
    fn initialize(&mut self) {}

    /** Optional method called after all historical events have been applied */
    fn after_rebuild(&mut self) {}

    /** Name of the projection for logging/debugging */
    fn name(&self) -> &str;
}
pub struct ProjectionApi {
    projection_manager: ProjectionManager,
}

/// API for event-related operations
pub struct EventApi {
    store: Arc<Mutex<EventStore>>,
}
impl Default for CoreApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreApi {
    /// Create a new instance of the logic API
    pub fn new() -> Self {
        // Create the event store
        let (event_store, event_sender) = create_event_store();

        // Create the person repository
        let repo = VecRepository::<PersonId, Person>::new();

        // Create the person service
        let person_service = Arc::new(Mutex::new(PersonService::new(repo, event_sender.clone())));

        // Create the projection manager
        let projection_manager = ProjectionManager::new(event_store.clone());

        // Register the location occupancy projection
        let location_projection =
            projection_manager.register_projection(LocationOccupancyProjection::new());

        // Create the terrain repository
        let terrain_repo = VecRepository::<TerrainId, Terrain>::new();
        let terrain_type_repo = VecRepository::<TerrainTypeId, TerrainType>::new();
        // Create the terrain service
        let terrain_service = Arc::new(Mutex::new(TerrainService::new(
            terrain_repo,
            terrain_type_repo,
            event_sender.clone(),
        )));
        // Give the projections a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(50));

        CoreApi {
            person: PersonApi {
                service: person_service,
            },
            location: LocationApi {
                projection: location_projection,
            },
            event: EventApi { store: event_store },
            projection: ProjectionApi { projection_manager },
            terrain: TerrainApi {
                service: terrain_service,
            },
        }
    }

    /// Access person-related operations
    pub fn person(&self) -> &PersonApi {
        &self.person
    }

    /// Access location-related queries
    pub fn location(&self) -> &LocationApi {
        &self.location
    }

    /// Access event-related operations
    pub fn event(&self) -> &EventApi {
        &self.event
    }
    pub fn projection(&self) -> &ProjectionApi {
        &self.projection
    }

    /// Access terrain-related operations
    pub fn terrain(&self) -> &TerrainApi {
        &self.terrain
    }
}
