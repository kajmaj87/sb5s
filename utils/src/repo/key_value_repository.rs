use crate::repo::Repository;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub enum KeyValueRepositoryError {
    NotFound,
    OperationNotSupported(String),
}

pub struct KeyValueRepository<ID, T>
where
    ID: Hash + Eq + Clone,
{
    data: HashMap<ID, T>,
}

impl<ID, T> KeyValueRepository<ID, T>
where
    ID: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        KeyValueRepository {
            data: HashMap::new(),
        }
    }

    pub fn with_capacity<F>(capacity: usize) -> Self
    where
        F: FnMut() -> ID + 'static,
    {
        KeyValueRepository {
            data: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn contains(&self, id: &ID) -> bool {
        self.data.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ID, &T)> {
        self.data.iter()
    }

    pub fn ids(&self) -> impl Iterator<Item = &ID> + '_ {
        self.data.keys()
    }

    // Insert with a specific ID (useful for deserialization)
    pub fn insert(&mut self, id: ID, entity: T) -> Option<T> {
        self.data.insert(id, entity)
    }
}

impl<ID, T: Clone> Repository<ID, T> for KeyValueRepository<ID, T>
where
    ID: Hash + Eq + Clone,
{
    type Error = KeyValueRepositoryError;

    fn get(&self, id: ID) -> Result<T, Self::Error> {
        match self.data.get(&id) {
            Some(entity) => Ok(entity.clone()),
            None => Err(KeyValueRepositoryError::NotFound),
        }
    }

    fn add(&mut self, entity: T) -> Result<ID, Self::Error> {
        Err(KeyValueRepositoryError::OperationNotSupported(
            "Add".to_string(),
        ))
    }

    fn contains(&self, id: ID) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(&id))
    }

    fn insert(&mut self, id: ID, entity: T) -> Result<T, Self::Error> {
        self.data.insert(id, entity.clone());
        Ok(entity)
    }

    fn remove(&mut self, id: ID) -> Result<T, Self::Error> {
        match self.data.remove(&id) {
            Some(entity) => Ok(entity),
            None => Err(KeyValueRepositoryError::NotFound),
        }
    }

    fn update(&mut self, id: ID, entity: T) -> Result<T, Self::Error> {
        if !self.data.contains_key(&id) {
            return Err(KeyValueRepositoryError::NotFound);
        }

        // We know the key exists, so this will never panic
        let old = self.data.insert(id, entity).unwrap();
        Ok(old)
    }

    fn get_all(&self) -> Result<Vec<T>, Self::Error> {
        let entities: Vec<T> = self.data.values().cloned().collect();
        Ok(entities)
    }

    fn create<F>(&mut self, entity_factory: F) -> Result<T, Self::Error>
    where
        F: FnOnce(ID) -> T,
    {
        Err(KeyValueRepositoryError::OperationNotSupported(
            "Create".to_string(),
        ))
    }
}
