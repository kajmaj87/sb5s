use crate::domain::entity::person::PersonId;
use crate::domain::value_object::location::Location;

#[derive(Debug, Clone, PartialEq)]
pub enum PersonEvent {
    PersonCreated {
        person_id: PersonId,
        name: String,
        location: Location,
    },
    PersonMoved {
        person_id: PersonId,
        from_location: Location,
        to_location: Location,
    },
}
