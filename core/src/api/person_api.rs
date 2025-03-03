use crate::domain::entity::person::{Person, PersonId};
use crate::domain::value_object::location::Location;
use crate::PersonApi;

impl PersonApi {
    /// Create a new person at the specified location
    pub fn create(&self, name: String, x: i32, y: i32) -> Result<Person, String> {
        let location = Location { x, y };
        self.service
            .lock()
            .unwrap()
            .create_person(name, location)
            .map_err(|e| format!("Failed to create person: {:?}", e))
    }

    /// Move a person to a new location
    pub fn move_to(&self, person_id: u32, x: i32, y: i32) -> Result<Person, String> {
        let location = Location { x, y };
        self.service
            .lock()
            .unwrap()
            .move_person(PersonId(person_id), location)
            .map_err(|e| format!("Failed to move person: {:?}", e))
    }

    /// Get a person by ID
    pub fn get(&self, person_id: u32) -> Result<Person, String> {
        self.service
            .lock()
            .unwrap()
            .get_person(PersonId(person_id))
            .map_err(|e| format!("Failed to get person: {:?}", e))
    }

    /// Get all persons
    pub fn get_all(&self) -> Result<Vec<Person>, String> {
        self.service
            .lock()
            .unwrap()
            .get_all_persons()
            .map_err(|e| format!("Failed to get all persons: {:?}", e))
    }
}
