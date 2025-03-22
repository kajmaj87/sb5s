use crate::domain::event::person_event::PersonEvent;

pub(crate) mod person_event;

#[derive(Debug, Clone, PartialEq)]
pub enum DomainEvent {
    Person(PersonEvent),
    // Other event types can be added here
}
