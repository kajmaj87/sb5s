mod vec_repository;
pub(crate) trait Repository<ID, Entity> {
    type Error;
    fn get(&self, id: ID) -> Result<Entity, Self::Error>;
    fn add(&mut self, entity: Entity) -> Result<ID, Self::Error>;
    fn remove(&mut self, id: ID) -> Result<Entity, Self::Error>;
    fn update(&mut self, id: ID, entity: Entity) -> Result<Entity, Self::Error>;
    fn get_all(&self) -> Result<Vec<Entity>, Self::Error>;
    fn create<F>(&mut self, entity_factory: F) -> Result<Entity, Self::Error>
    where
        F: FnOnce(ID) -> Entity;
}

pub(crate) trait NumericId: Copy + Eq + std::fmt::Debug {
    fn value(&self) -> u32;
    fn from_value(value: u32) -> Self;
}

pub(crate) use vec_repository::VecRepository;
