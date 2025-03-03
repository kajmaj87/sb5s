mod event_api;
mod location_api;
mod person_api;

use crate::domain::entity::person::{Person, PersonId};
use crate::domain::service::person_service::PersonService;
use crate::infrastructure::event_store::{create_event_store, EventStore};
use crate::infrastructure::projection::{LocationOccupancyProjection, ProjectionManager};
use crate::repo::VecRepository;
use std::sync::{Arc, Mutex};

/// Main API facade for the core module
pub struct CoreApi {
    person: PersonApi,
    location: LocationApi,
    event: EventApi,
}
/// API for person-related operations
pub struct PersonApi {
    service: Arc<Mutex<PersonService<VecRepository<PersonId, Person>>>>,
}

/// API for location-related queries
pub struct LocationApi {
    projection: Arc<Mutex<LocationOccupancyProjection>>,
}

/// API for event-related operations
pub struct EventApi {
    store: Arc<Mutex<EventStore>>,
}
impl CoreApi {
    /// Create a new instance of the core API
    pub fn new() -> Self {
        // Create the event store
        let (event_store, event_sender) = create_event_store();

        // Create the person repository
        let repo = VecRepository::<PersonId, Person>::new();

        // Create the person service
        let person_service = Arc::new(Mutex::new(PersonService::new(repo, event_sender)));

        // Create the projection manager
        let projection_manager = ProjectionManager::new(event_store.clone());

        // Register the location occupancy projection
        let location_projection =
            projection_manager.register_projection(LocationOccupancyProjection::new());

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
}
