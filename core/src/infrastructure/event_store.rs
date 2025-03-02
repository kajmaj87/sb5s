use crate::domain::event::DomainEvent;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Stores all domain events and allows subscribers to receive them
pub(crate) struct EventStore {
    events: Vec<DomainEvent>,
    receiver: Receiver<DomainEvent>,
    subscribers: Vec<Sender<DomainEvent>>,
}

impl EventStore {
    /// Create a new event store with a receiver for incoming events
    pub fn new(receiver: Receiver<DomainEvent>) -> Self {
        EventStore {
            events: Vec::new(),
            receiver,
            subscribers: Vec::new(),
        }
    }

    /// Add a new subscriber that will receive future events
    pub fn subscribe(&mut self) -> Receiver<DomainEvent> {
        let (sender, receiver) = mpsc::channel();
        self.subscribers.push(sender);
        receiver
    }

    /// Get all historical events for rebuilding projections
    pub fn get_all_events(&self) -> Vec<DomainEvent> {
        self.events.clone()
    }

    /// Get the total number of stored events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Start processing events in a background thread
    pub fn start_processing(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            println!("Event store started processing events");

            while let Ok(event) = self.receiver.recv() {
                println!("Event received: {:?}", event);

                // Store the event
                self.events.push(event.clone());

                // Notify all subscribers
                self.subscribers
                    .retain(|sender| sender.send(event.clone()).is_ok());
            }

            println!("Event store stopped processing events");
        })
    }
}
/// Create a new event store and return a sender for publishing events to it
pub fn create_event_store() -> (Arc<Mutex<EventStore>>, Sender<DomainEvent>) {
    let (sender, receiver) = mpsc::channel();
    let event_store = EventStore::new(receiver);

    let event_store_arc = Arc::new(Mutex::new(event_store));

    let event_store_for_thread = event_store_arc.clone();

    thread::spawn(move || {
        let event_store = {
            let mut guard = event_store_for_thread.lock().unwrap();
            std::mem::replace(&mut *guard, EventStore::new(mpsc::channel().1))
        };

        event_store.start_processing().join().unwrap();
    });

    (event_store_arc, sender)
}

/// Helper function to publish an event to a channel
pub fn publish_event<T>(sender: &Sender<T>, event: T) {
    if let Err(e) = sender.send(event) {
        eprintln!("Failed to publish event: {:?}", e);
    }
}
