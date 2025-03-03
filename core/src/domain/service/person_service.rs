use crate::domain::entity::person::Person;
use crate::domain::entity::person::PersonId;
use crate::domain::event::person_event::PersonEvent;
use crate::domain::event::DomainEvent;
use crate::domain::value_object::location::Location;
use crate::infrastructure::event_store::publish_event;
use crate::repo::Repository;
use std::sync::mpsc::Sender;

pub struct PersonService<R: Repository<PersonId, Person>> {
    repository: R,
    event_sender: Sender<DomainEvent>,
}

impl<R: Repository<PersonId, Person>> PersonService<R> {
    pub fn new(repository: R, event_sender: Sender<DomainEvent>) -> Self {
        PersonService {
            repository,
            event_sender,
        }
    }

    // Create a new person and emit a PersonCreated event
    pub fn create_person(&mut self, name: String, location: Location) -> Result<Person, R::Error> {
        // Create the person using the repository
        let person = self.repository.create(|id| Person {
            id,
            name: name.clone(),
            location: location.clone(),
        })?;

        // Emit the PersonCreated event
        let event = PersonEvent::PersonCreated {
            person_id: person.id,
            name,
            location,
        };

        publish_event(&self.event_sender, DomainEvent::Person(event));

        Ok(person)
    }

    // Move a person to a new location and emit a PersonMoved event
    pub fn move_person(
        &mut self,
        person_id: PersonId,
        new_location: Location,
    ) -> Result<Person, R::Error> {
        // Get the current person
        let current_person = self.repository.get(person_id)?;
        let old_location = current_person.location.clone();

        // Create an updated person with the new location
        let updated_person = Person {
            id: person_id,
            name: current_person.name,
            location: new_location.clone(),
        };

        // Update the person in the repository
        self.repository.update(person_id, updated_person.clone())?;

        // Emit the PersonMoved event
        let event = PersonEvent::PersonMoved {
            person_id,
            from_location: old_location,
            to_location: new_location,
        };

        publish_event(&self.event_sender, DomainEvent::Person(event));

        Ok(updated_person)
    }

    // Get a person by ID
    pub fn get_person(&self, person_id: PersonId) -> Result<Person, R::Error> {
        self.repository.get(person_id)
    }

    // Get all persons
    pub fn get_all_persons(&self) -> Result<Vec<Person>, R::Error> {
        self.repository.get_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::VecRepository;
    use std::sync::mpsc;

    #[test]
    fn test_create_person() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Create a person
        let location = Location { x: 10, y: 20 };
        let person = service
            .create_person("Alice".to_string(), location.clone())
            .unwrap();

        // Verify the person was created correctly
        assert_eq!(person.id, PersonId(0));
        assert_eq!(person.name, "Alice");
        assert_eq!(person.location, location);

        // Verify an event was sent
        let event = receiver.recv().unwrap();
        if let DomainEvent::Person(PersonEvent::PersonCreated {
            person_id,
            name,
            location: event_location,
        }) = event
        {
            assert_eq!(person_id, PersonId(0));
            assert_eq!(name, "Alice");
            assert_eq!(event_location, location);
        } else {
            panic!("Expected PersonCreated event");
        }
    }

    #[test]
    fn test_move_person() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();

        // Add a person directly to the repository
        let initial_location = Location { x: 10, y: 20 };
        let person = Person {
            id: PersonId(0),
            name: "Bob".to_string(),
            location: initial_location.clone(),
        };
        repo.add(person).unwrap();

        let mut service = PersonService::new(repo, sender);

        // Move the person
        let new_location = Location { x: 30, y: 40 };
        let updated_person = service
            .move_person(PersonId(0), new_location.clone())
            .unwrap();

        // Verify the person was moved correctly
        assert_eq!(updated_person.id, PersonId(0));
        assert_eq!(updated_person.name, "Bob");
        assert_eq!(updated_person.location, new_location);

        // Verify an event was sent
        let event = receiver.recv().unwrap();
        if let DomainEvent::Person(PersonEvent::PersonMoved {
            person_id,
            from_location,
            to_location,
        }) = event
        {
            assert_eq!(person_id, PersonId(0));
            assert_eq!(from_location, initial_location);
            assert_eq!(to_location, new_location);
        } else {
            panic!("Expected PersonMoved event");
        }
    }

    #[test]
    fn test_get_person() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();

        // Add a person directly to the repository
        let location = Location { x: 10, y: 20 };
        let person = Person {
            id: PersonId(0),
            name: "Charlie".to_string(),
            location: location.clone(),
        };
        repo.add(person.clone()).unwrap();

        let service = PersonService::new(repo, sender);

        // Get the person
        let retrieved_person = service.get_person(PersonId(0)).unwrap();

        // Verify the correct person was retrieved
        assert_eq!(retrieved_person, person);

        // Verify no events were sent (get operations don't emit events)
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_get_all_persons() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();

        // Add multiple persons directly to the repository
        let person1 = Person {
            id: PersonId(0),
            name: "Dave".to_string(),
            location: Location { x: 10, y: 20 },
        };
        let person2 = Person {
            id: PersonId(1),
            name: "Eve".to_string(),
            location: Location { x: 30, y: 40 },
        };

        repo.add(person1.clone()).unwrap();
        repo.add(person2.clone()).unwrap();

        let service = PersonService::new(repo, sender);

        // Get all persons
        let all_persons = service.get_all_persons().unwrap();

        // Verify all persons were retrieved
        assert_eq!(all_persons.len(), 2);
        assert!(all_persons.contains(&person1));
        assert!(all_persons.contains(&person2));

        // Verify no events were sent (get operations don't emit events)
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_move_nonexistent_person() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Try to move a nonexistent person
        let result = service.move_person(PersonId(99), Location { x: 50, y: 60 });

        // Verify the operation failed
        assert!(result.is_err());

        // Verify no events were sent for failed operations
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_create_multiple_persons() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Create multiple persons
        let person1 = service
            .create_person("Frank".to_string(), Location { x: 10, y: 20 })
            .unwrap();

        let person2 = service
            .create_person("Grace".to_string(), Location { x: 30, y: 40 })
            .unwrap();

        // Verify the persons were created with sequential IDs
        assert_eq!(person1.id, PersonId(0));
        assert_eq!(person2.id, PersonId(1));

        // Verify events were sent for both creations
        let event1 = receiver.recv().unwrap();
        let event2 = receiver.recv().unwrap();

        // Check first event
        if let DomainEvent::Person(PersonEvent::PersonCreated {
            person_id, name, ..
        }) = event1
        {
            assert_eq!(person_id, PersonId(0));
            assert_eq!(name, "Frank");
        } else {
            panic!("Expected PersonCreated event");
        }

        // Check second event
        if let DomainEvent::Person(PersonEvent::PersonCreated {
            person_id, name, ..
        }) = event2
        {
            assert_eq!(person_id, PersonId(1));
            assert_eq!(name, "Grace");
        } else {
            panic!("Expected PersonCreated event");
        }

        // Verify no more events were sent
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_complex_workflow() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Create a person
        let person = service
            .create_person("Hannah".to_string(), Location { x: 10, y: 20 })
            .unwrap();

        // Move the person
        let updated_person = service
            .move_person(person.id, Location { x: 30, y: 40 })
            .unwrap();

        // Move the person again
        let final_person = service
            .move_person(updated_person.id, Location { x: 50, y: 60 })
            .unwrap();

        // Verify the final state
        assert_eq!(final_person.id, PersonId(0));
        assert_eq!(final_person.name, "Hannah");
        assert_eq!(final_person.location, Location { x: 50, y: 60 });

        // Verify all events were sent
        let event1 = receiver.recv().unwrap();
        let event2 = receiver.recv().unwrap();
        let event3 = receiver.recv().unwrap();

        // Check the event types
        assert!(matches!(
            event1,
            DomainEvent::Person(PersonEvent::PersonCreated { .. })
        ));
        assert!(matches!(
            event2,
            DomainEvent::Person(PersonEvent::PersonMoved { .. })
        ));
        assert!(matches!(
            event3,
            DomainEvent::Person(PersonEvent::PersonMoved { .. })
        ));

        // Check the last move event in detail
        if let DomainEvent::Person(PersonEvent::PersonMoved {
            person_id,
            from_location,
            to_location,
        }) = event3
        {
            assert_eq!(person_id, PersonId(0));
            assert_eq!(from_location, Location { x: 30, y: 40 });
            assert_eq!(to_location, Location { x: 50, y: 60 });
        } else {
            panic!("Expected PersonMoved event");
        }

        // Verify no more events were sent
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_create_person_with_same_name() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Create two persons with the same name but different locations
        let location1 = Location { x: 10, y: 20 };
        let location2 = Location { x: 30, y: 40 };

        let person1 = service
            .create_person("Duplicate".to_string(), location1.clone())
            .unwrap();
        let person2 = service
            .create_person("Duplicate".to_string(), location2.clone())
            .unwrap();

        // Verify they have different IDs
        assert_eq!(person1.id, PersonId(0));
        assert_eq!(person2.id, PersonId(1));

        // Verify they have the same name but different locations
        assert_eq!(person1.name, person2.name);
        assert_ne!(person1.location, person2.location);

        // Verify events were sent for both creations
        let event1 = receiver.recv().unwrap();
        let event2 = receiver.recv().unwrap();

        assert!(matches!(
            event1,
            DomainEvent::Person(PersonEvent::PersonCreated { .. })
        ));
        assert!(matches!(
            event2,
            DomainEvent::Person(PersonEvent::PersonCreated { .. })
        ));
    }

    #[test]
    fn test_move_person_to_same_location() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();

        // Add a person directly to the repository
        let location = Location { x: 10, y: 20 };
        let person = Person {
            id: PersonId(0),
            name: "Stationary".to_string(),
            location: location.clone(),
        };
        repo.add(person).unwrap();

        let mut service = PersonService::new(repo, sender);

        // Move the person to the same location
        let updated_person = service.move_person(PersonId(0), location.clone()).unwrap();

        // Verify the person's location didn't change
        assert_eq!(updated_person.location, location);

        // Verify an event was still sent (even though the location didn't change)
        let event = receiver.recv().unwrap();
        if let DomainEvent::Person(PersonEvent::PersonMoved {
            person_id,
            from_location,
            to_location,
        }) = event
        {
            assert_eq!(person_id, PersonId(0));
            assert_eq!(from_location, location);
            assert_eq!(to_location, location);
        } else {
            panic!("Expected PersonMoved event");
        }
    }

    #[test]
    fn test_get_nonexistent_person() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let repo = VecRepository::<PersonId, Person>::new();
        let service = PersonService::new(repo, sender);

        // Try to get a nonexistent person
        let result = service.get_person(PersonId(99));

        // Verify the operation failed
        assert!(result.is_err());

        // Verify no events were sent
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_create_person_after_removing_one() {
        // Setup
        let (sender, receiver) = mpsc::channel();
        let mut repo = VecRepository::<PersonId, Person>::new();

        // Add a person directly to the repository
        let person = Person {
            id: PersonId(0),
            name: "Temporary".to_string(),
            location: Location { x: 10, y: 20 },
        };
        repo.add(person).unwrap();

        // Remove the person
        repo.remove(PersonId(0)).unwrap();

        let mut service = PersonService::new(repo, sender);

        // Create a new person
        let new_person = service
            .create_person("Replacement".to_string(), Location { x: 30, y: 40 })
            .unwrap();

        // Verify the new person has ID 1 (not reusing the removed ID 0)
        assert_eq!(new_person.id, PersonId(1));

        // Verify an event was sent
        let event = receiver.recv().unwrap();
        if let DomainEvent::Person(PersonEvent::PersonCreated {
            person_id, name, ..
        }) = event
        {
            assert_eq!(person_id, PersonId(1));
            assert_eq!(name, "Replacement");
        } else {
            panic!("Expected PersonCreated event");
        }
    }

    #[test]
    fn test_channel_closed() {
        // Setup - create a channel and drop the receiver to close it
        let (sender, _receiver) = mpsc::channel();
        let repo = VecRepository::<PersonId, Person>::new();
        let mut service = PersonService::new(repo, sender);

        // Create a person - this should not panic even though the channel is closed
        let result = service.create_person("Undelivered".to_string(), Location { x: 10, y: 20 });

        // The operation should still succeed even though the event couldn't be sent
        assert!(result.is_ok());
    }
}
