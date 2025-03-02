use crate::domain::event::DomainEvent;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

struct EventStore {
    events: Vec<DomainEvent>,
    receiver: Receiver<DomainEvent>,
}
impl EventStore {
    fn new(receiver: Receiver<DomainEvent>) -> Self {
        EventStore {
            events: Vec::new(),
            receiver,
        }
    }

    // Start processing events in a background thread
    fn start_processing(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            println!("Event store started processing events");

            while let Ok(event) = self.receiver.recv() {
                println!("Event received: {:?}", event);
                self.events.push(event);

                // Notify subscribers or perform other processing
                // ...
            }

            println!("Event store stopped processing events");
        })
    }
}

pub fn publish_event<T>(sender: &Sender<T>, event: T) {
    if let Err(e) = sender.send(event) {
        eprintln!("Failed to publish event: {:?}", e);
    }
}
