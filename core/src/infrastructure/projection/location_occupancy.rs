use crate::domain::entity::person::PersonId;
use crate::domain::event::person_event::PersonEvent;
use crate::domain::event::DomainEvent;
use crate::domain::value_object::location::Location;
use crate::infrastructure::projection::Projection;
use std::collections::HashMap;

/// Projection that tracks which people are at each location
pub struct LocationOccupancyProjection {
    occupancy: HashMap<Location, Vec<PersonId>>,
}

impl LocationOccupancyProjection {
    /// Creates a new empty location occupancy projection
    pub fn new() -> Self {
        LocationOccupancyProjection {
            occupancy: HashMap::new(),
        }
    }

    fn add_person_to_location(&mut self, person_id: PersonId, location: Location) {
        self.occupancy
            .entry(location)
            .or_insert_with(Vec::new)
            .push(person_id);
    }

    fn remove_person_from_location(&mut self, person_id: PersonId, location: &Location) {
        if let Some(people) = self.occupancy.get_mut(location) {
            people.retain(|&id| id != person_id);

            if people.is_empty() {
                self.occupancy.remove(location);
            }
        }
    }

    /// Returns all people currently at the specified location
    pub fn get_people_at_location(&self, location: &Location) -> Vec<PersonId> {
        self.occupancy.get(location).cloned().unwrap_or_default()
    }

    /// Returns all locations that currently have at least one person
    pub fn get_occupied_locations(&self) -> Vec<Location> {
        self.occupancy.keys().cloned().collect()
    }

    /// Returns the total number of locations that have at least one person
    pub fn get_occupied_location_count(&self) -> usize {
        self.occupancy.len()
    }

    /// Returns the location with the most people, along with the count
    /// Returns None if no locations are occupied
    pub fn get_most_crowded_location(&self) -> Option<(Location, usize)> {
        self.occupancy
            .iter()
            .map(|(location, people)| (location.clone(), people.len()))
            .max_by_key(|&(_, count)| count)
    }
}

impl Projection for LocationOccupancyProjection {
    fn apply(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::Person(PersonEvent::PersonCreated {
                person_id,
                location,
                ..
            }) => {
                self.add_person_to_location(*person_id, location.clone());
            }
            DomainEvent::Person(PersonEvent::PersonMoved {
                person_id,
                from_location,
                to_location,
            }) => {
                self.remove_person_from_location(*person_id, from_location);
                self.add_person_to_location(*person_id, to_location.clone());
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "LocationOccupancyProjection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_person_created_event(id: u32, x: i32, y: i32) -> DomainEvent {
        DomainEvent::Person(PersonEvent::PersonCreated {
            person_id: PersonId(id),
            name: format!("Person {}", id),
            location: Location { x, y },
        })
    }

    fn create_person_moved_event(
        id: u32,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    ) -> DomainEvent {
        DomainEvent::Person(PersonEvent::PersonMoved {
            person_id: PersonId(id),
            from_location: Location {
                x: from_x,
                y: from_y,
            },
            to_location: Location { x: to_x, y: to_y },
        })
    }

    #[test]
    fn test_new_projection_is_empty() {
        let projection = LocationOccupancyProjection::new();

        assert_eq!(projection.get_occupied_location_count(), 0);
        assert!(projection.get_occupied_locations().is_empty());
        assert!(projection.get_most_crowded_location().is_none());
    }

    #[test]
    fn test_apply_person_created_event() {
        let mut projection = LocationOccupancyProjection::new();
        let location = Location { x: 10, y: 20 };

        projection.apply(&create_person_created_event(1, 10, 20));

        assert_eq!(projection.get_occupied_location_count(), 1);
        assert_eq!(projection.get_occupied_locations(), vec![location.clone()]);
        assert_eq!(
            projection.get_people_at_location(&location),
            vec![PersonId(1)]
        );
    }

    #[test]
    fn test_apply_person_moved_event() {
        let mut projection = LocationOccupancyProjection::new();
        let location1 = Location { x: 10, y: 20 };
        let location2 = Location { x: 30, y: 40 };

        // Create a person at location1
        projection.apply(&create_person_created_event(1, 10, 20));

        // Move the person to location2
        projection.apply(&create_person_moved_event(1, 10, 20, 30, 40));

        // Verify the person is now at location2 and not at location1
        assert_eq!(projection.get_people_at_location(&location1), vec![]);
        assert_eq!(
            projection.get_people_at_location(&location2),
            vec![PersonId(1)]
        );

        // Verify location1 is no longer in the occupied locations
        assert_eq!(projection.get_occupied_locations(), vec![location2]);
        assert_eq!(projection.get_occupied_location_count(), 1);
    }

    #[test]
    fn test_multiple_people_at_same_location() {
        let mut projection = LocationOccupancyProjection::new();
        let location = Location { x: 10, y: 20 };

        // Create three people at the same location
        projection.apply(&create_person_created_event(1, 10, 20));
        projection.apply(&create_person_created_event(2, 10, 20));
        projection.apply(&create_person_created_event(3, 10, 20));

        // Verify all three people are at the location
        let people = projection.get_people_at_location(&location);
        assert_eq!(people.len(), 3);
        assert!(people.contains(&PersonId(1)));
        assert!(people.contains(&PersonId(2)));
        assert!(people.contains(&PersonId(3)));

        // Verify this is the most crowded location
        let (crowded_location, count) = projection.get_most_crowded_location().unwrap();
        assert_eq!(crowded_location, location);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_people_at_different_locations() {
        let mut projection = LocationOccupancyProjection::new();
        let location1 = Location { x: 10, y: 20 };
        let location2 = Location { x: 30, y: 40 };
        let location3 = Location { x: 50, y: 60 };

        // Create people at different locations
        projection.apply(&create_person_created_event(1, 10, 20));
        projection.apply(&create_person_created_event(2, 30, 40));
        projection.apply(&create_person_created_event(3, 50, 60));

        // Verify each person is at their respective location
        assert_eq!(
            projection.get_people_at_location(&location1),
            vec![PersonId(1)]
        );
        assert_eq!(
            projection.get_people_at_location(&location2),
            vec![PersonId(2)]
        );
        assert_eq!(
            projection.get_people_at_location(&location3),
            vec![PersonId(3)]
        );

        // Verify all locations are occupied
        let occupied_locations = projection.get_occupied_locations();
        assert_eq!(occupied_locations.len(), 3);
        assert!(occupied_locations.contains(&location1));
        assert!(occupied_locations.contains(&location2));
        assert!(occupied_locations.contains(&location3));
    }

    #[test]
    fn test_most_crowded_location() {
        let mut projection = LocationOccupancyProjection::new();

        // Create different numbers of people at different locations
        projection.apply(&create_person_created_event(1, 10, 20));
        projection.apply(&create_person_created_event(2, 30, 40));
        projection.apply(&create_person_created_event(3, 30, 40));
        projection.apply(&create_person_created_event(4, 30, 40));
        projection.apply(&create_person_created_event(5, 50, 60));
        projection.apply(&create_person_created_event(6, 50, 60));

        // Verify the most crowded location
        let (crowded_location, count) = projection.get_most_crowded_location().unwrap();
        assert_eq!(crowded_location, Location { x: 30, y: 40 });
        assert_eq!(count, 3);
    }

    #[test]
    fn test_moving_last_person_from_location() {
        let mut projection = LocationOccupancyProjection::new();
        let location1 = Location { x: 10, y: 20 };
        let location2 = Location { x: 30, y: 40 };

        // Create a person at location1
        projection.apply(&create_person_created_event(1, 10, 20));

        // Move the person to location2
        projection.apply(&create_person_moved_event(1, 10, 20, 30, 40));

        // Verify location1 is no longer in the occupied locations
        assert!(!projection.get_occupied_locations().contains(&location1));
        assert_eq!(projection.get_occupied_location_count(), 1);
    }

    #[test]
    fn test_complex_scenario() {
        let mut projection = LocationOccupancyProjection::new();

        // Create people at different locations
        projection.apply(&create_person_created_event(1, 10, 20)); // Person 1 at (10,20)
        projection.apply(&create_person_created_event(2, 30, 40)); // Person 2 at (30,40)
        projection.apply(&create_person_created_event(3, 10, 20)); // Person 3 at (10,20)

        // Move people around
        projection.apply(&create_person_moved_event(1, 10, 20, 30, 40)); // Person 1 moves to (30,40)
        projection.apply(&create_person_moved_event(2, 30, 40, 50, 60)); // Person 2 moves to (50,60)

        // Verify final state
        assert_eq!(
            projection.get_people_at_location(&Location { x: 10, y: 20 }),
            vec![PersonId(3)]
        );
        assert_eq!(
            projection.get_people_at_location(&Location { x: 30, y: 40 }),
            vec![PersonId(1)]
        );
        assert_eq!(
            projection.get_people_at_location(&Location { x: 50, y: 60 }),
            vec![PersonId(2)]
        );

        assert_eq!(projection.get_occupied_location_count(), 3);
    }
}
