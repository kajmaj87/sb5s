use crate::domain::event::DomainEvent;
use parking_lot::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread;

/// Stores all domain events and allows subscribers to receive them
pub struct EventStore {
    events: Vec<DomainEvent>,
    subscribers: Vec<Sender<DomainEvent>>,
}

impl EventStore {
    pub fn new() -> Self {
        EventStore {
            events: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> Receiver<DomainEvent> {
        let (sender, receiver) = mpsc::channel();
        println!(
            "New subscriber added (total: {})",
            self.subscribers.len() + 1
        );
        self.subscribers.push(sender);
        receiver
    }

    pub fn publish(&mut self, event: DomainEvent) {
        // println!("Event received: {:?}", event);
        // Store the event
        self.events.push(event.clone());

        // Notify all subscribers and remove dead ones
        self.subscribers.retain(|sender| {
            let result = sender.send(event.clone());
            if result.is_err() {
                println!("Failed to send to subscriber - removing");
            }
            result.is_ok()
        });
    }

    pub fn get_all_events(&self) -> Vec<DomainEvent> {
        self.events.clone()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

pub fn create_event_store() -> (Arc<Mutex<EventStore>>, Sender<DomainEvent>) {
    let event_store = Arc::new(Mutex::new(EventStore::new()));
    let event_store_clone = event_store.clone();

    // Create a channel for publishing events
    let (sender, receiver) = mpsc::channel();

    // Spawn a thread that processes events
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            let mut store = event_store_clone.lock();
            store.publish(event);
        }
    });

    (event_store, sender)
}

/// Helper function to publish an event to a channel
pub fn publish_event<T>(sender: &Sender<T>, event: T) {
    if let Err(e) = sender.send(event) {
        eprintln!("Failed to publish event: {:?}", e);
    }
}
