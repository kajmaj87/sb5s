mod key_value_repository;
mod vec_repository;

pub trait Repository<ID, Entity> {
    type Error;
    fn get(&self, id: ID) -> Result<Entity, Self::Error>;
    fn add(&mut self, entity: Entity) -> Result<ID, Self::Error>;
    fn contains(&self, id: ID) -> Result<bool, Self::Error>;
    fn insert(&mut self, id: ID, entity: Entity) -> Result<Entity, Self::Error>;
    fn remove(&mut self, id: ID) -> Result<Entity, Self::Error>;
    fn update(&mut self, id: ID, entity: Entity) -> Result<Entity, Self::Error>;
    fn get_all(&self) -> Result<Vec<Entity>, Self::Error>;
    fn create<F>(&mut self, entity_factory: F) -> Result<Entity, Self::Error>
    where
        F: FnOnce(ID) -> Entity;
}

pub trait NumericId: Copy + Eq + std::fmt::Debug {
    fn value(&self) -> u32;
    fn from_value(value: u32) -> Self;
    fn next(&self) -> Self {
        Self::from_value(self.value() + 1)
    }
}

pub use key_value_repository::KeyValueRepository;
pub use vec_repository::VecRepository;
