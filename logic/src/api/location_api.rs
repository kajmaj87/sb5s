use crate::domain::value_object::location::Location;
use crate::LocationApi;

impl LocationApi {
    /// Get all people at a specific location
    pub fn get_people_at(&self, x: i32, y: i32) -> Vec<u32> {
        let location = Location { x, y };
        self.projection
            .lock()
            .unwrap()
            .get_people_at_location(&location)
            .into_iter()
            .map(|id| id.0)
            .collect()
    }

    /// Get all occupied locations
    pub fn get_occupied(&self) -> Vec<(i32, i32)> {
        self.projection
            .lock()
            .unwrap()
            .get_occupied_locations()
            .into_iter()
            .map(|loc| (loc.x, loc.y))
            .collect()
    }

    /// Get the most crowded location
    pub fn most_crowded(&self) -> Option<(i32, i32, usize)> {
        self.projection
            .lock()
            .unwrap()
            .get_most_crowded_location()
            .map(|(loc, count)| (loc.x, loc.y, count))
    }

    /// Get the number of occupied locations
    pub fn occupied_count(&self) -> usize {
        self.projection
            .lock()
            .unwrap()
            .get_occupied_location_count()
    }
}
