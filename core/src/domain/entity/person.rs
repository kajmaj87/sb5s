use crate::domain::value_object::location::Location;
use crate::repo::NumericId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersonId(pub u32);
impl NumericId for PersonId {
    fn value(&self) -> u32 {
        self.0
    }

    fn from_value(value: u32) -> Self {
        PersonId(value)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub location: Location,
}
