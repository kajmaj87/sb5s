pub(crate) mod location_occupancy;
use crate::infrastructure::event_store::EventStore;
use crate::Projection;
pub use location_occupancy::LocationOccupancyProjection;
use parking_lot::Mutex;

/** Projection manager that handles creating and rebuilding projections */
pub struct ProjectionManager {
    event_store: std::sync::Arc<Mutex<EventStore>>,
}

impl ProjectionManager {
    pub fn new(event_store: std::sync::Arc<Mutex<EventStore>>) -> Self {
        ProjectionManager { event_store }
    }

    // Register a new projection, rebuild it from history, and start processing live events
    pub fn register_projection<P: Projection>(&self, projection: P) -> std::sync::Arc<Mutex<P>> {
        let projection_arc = std::sync::Arc::new(Mutex::new(projection));
        let projection_clone = projection_arc.clone();

        // Get a receiver for new events
        let receiver = {
            let mut store = self.event_store.lock();
            store.subscribe()
        };

        // Get all historical events
        let historical_events = {
            let store = self.event_store.lock();
            store.get_all_events()
        };

        // Start a thread to rebuild from history and then process live events
        std::thread::spawn(move || {
            let mut projection = projection_clone.lock();

            println!("Initializing projection: {}", projection.name());
            projection.initialize();

            println!(
                "Rebuilding projection {} from {} historical events",
                projection.name(),
                historical_events.len()
            );

            // Apply all historical events
            for event in &historical_events {
                projection.apply(event);
            }

            println!("Finished rebuilding projection: {}", projection.name());
            projection.after_rebuild();

            // Release the lock before starting to process live events
            drop(projection);

            println!(
                "Starting to process live events for projection: {}",
                projection_clone.lock().name()
            );

            // Process live events
            while let Ok(event) = receiver.recv() {
                let mut projection = projection_clone.lock();
                projection.apply(&event);
            }

            println!(
                "Stopped processing events for projection: {}",
                projection_clone.lock().name()
            );
        });

        projection_arc
    }
}
