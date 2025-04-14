use crate::domain::value_object::location::Location;
use crate::NumericId;
use utils_derive::NumericId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, NumericId)]
pub struct PersonId(pub u32);
#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub location: Location,
}
