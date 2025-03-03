use crate::EventApi;

impl EventApi {
    /// Get the total number of events in the event store
    pub fn count(&self) -> usize {
        self.store.lock().unwrap().event_count()
    }
}
