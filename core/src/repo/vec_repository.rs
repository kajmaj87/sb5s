use crate::repo::{NumericId, Repository};

#[derive(Debug)]
pub(crate) enum VecRepositoryError {
    NotFound,
    InvalidIndex,
}

pub(crate) struct VecRepository<ID: NumericId, T> {
    data: Vec<Option<T>>,
    _id_type: std::marker::PhantomData<ID>,
}

impl<ID: NumericId, T> VecRepository<ID, T> {
    pub(crate) fn new() -> Self {
        VecRepository {
            data: Vec::new(),
            _id_type: Default::default(),
        }
    }
}

impl<ID: NumericId, T: Clone> Repository<ID, T> for VecRepository<ID, T> {
    type Error = VecRepositoryError;

    fn get(&self, id: ID) -> Result<T, Self::Error> {
        let index = id.value() as usize;
        if index >= self.data.len() {
            return Err(VecRepositoryError::NotFound);
        }

        match &self.data[index] {
            Some(entity) => Ok(entity.clone()),
            None => Err(VecRepositoryError::NotFound),
        }
    }

    fn add(&mut self, entity: T) -> Result<ID, Self::Error> {
        let id = ID::from_value(self.data.len() as u32);
        self.data.push(Some(entity));
        Ok(id)
    }

    fn remove(&mut self, id: ID) -> Result<T, Self::Error> {
        let index = id.value() as usize;
        if index >= self.data.len() {
            return Err(VecRepositoryError::NotFound);
        }

        match self.data[index].take() {
            Some(entity) => Ok(entity),
            None => Err(VecRepositoryError::NotFound),
        }
    }

    fn update(&mut self, id: ID, entity: T) -> Result<T, Self::Error> {
        let index = id.value() as usize;
        if index >= self.data.len() {
            return Err(VecRepositoryError::NotFound);
        }

        match self.data[index].take() {
            Some(old_entity) => {
                self.data[index] = Some(entity);
                Ok(old_entity)
            }
            None => Err(VecRepositoryError::NotFound),
        }
    }

    fn get_all(&self) -> Result<Vec<T>, Self::Error> {
        let entities: Vec<T> = self
            .data
            .iter()
            .filter_map(|opt| opt.as_ref().map(|entity| entity.clone()))
            .collect();
        Ok(entities)
    }
    fn create<F>(&mut self, entity_factory: F) -> Result<T, Self::Error>
    where
        F: FnOnce(ID) -> T,
    {
        let id = ID::from_value(self.data.len() as u32);
        let entity = entity_factory(id);
        self.data.push(Some(entity.clone()));
        Ok(entity)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::NumericId;

    // Define a test ID type that implements NumericId
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestId(u32);

    impl NumericId for TestId {
        fn value(&self) -> u32 {
            self.0
        }

        fn from_value(value: u32) -> Self {
            TestId(value)
        }
    }

    // Helper function to create a repository for strings with TestId
    fn create_string_repo() -> VecRepository<TestId, String> {
        VecRepository::new()
    }

    #[test]
    fn test_add_and_get() {
        let mut repo = create_string_repo();

        // Add an entity
        let id = repo.add("test entity".to_string()).unwrap();
        assert_eq!(id.value(), 0);

        // Get the entity
        let entity = repo.get(id).unwrap();
        assert_eq!(entity, "test entity");
    }

    #[test]
    fn test_add_multiple() {
        let mut repo = create_string_repo();

        let id1 = repo.add("entity 1".to_string()).unwrap();
        let id2 = repo.add("entity 2".to_string()).unwrap();
        let id3 = repo.add("entity 3".to_string()).unwrap();

        assert_eq!(id1.value(), 0);
        assert_eq!(id2.value(), 1);
        assert_eq!(id3.value(), 2);

        assert_eq!(repo.get(id1).unwrap(), "entity 1");
        assert_eq!(repo.get(id2).unwrap(), "entity 2");
        assert_eq!(repo.get(id3).unwrap(), "entity 3");
    }

    #[test]
    fn test_get_nonexistent() {
        let repo = create_string_repo();

        // Try to get an entity with an ID that doesn't exist
        let result = repo.get(TestId(0));
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_remove() {
        let mut repo = create_string_repo();

        // Add and then remove an entity
        let id = repo.add("test entity".to_string()).unwrap();
        let removed = repo.remove(id).unwrap();

        assert_eq!(removed, "test entity");

        // Try to get the removed entity
        let result = repo.get(id);
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut repo = create_string_repo();

        // Try to remove an entity with an ID that doesn't exist
        let result = repo.remove(TestId(0));
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_remove_already_removed() {
        let mut repo = create_string_repo();

        // Add and remove an entity
        let id = repo.add("test entity".to_string()).unwrap();
        repo.remove(id).unwrap();

        // Try to remove it again
        let result = repo.remove(id);
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_update() {
        let mut repo = create_string_repo();

        // Add an entity
        let id = repo.add("original".to_string()).unwrap();

        // Update the entity
        let old = repo.update(id, "updated".to_string()).unwrap();

        assert_eq!(old, "original");
        assert_eq!(repo.get(id).unwrap(), "updated");
    }

    #[test]
    fn test_update_nonexistent() {
        let mut repo = create_string_repo();

        // Try to update an entity with an ID that doesn't exist
        let result = repo.update(TestId(0), "updated".to_string());
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_update_removed() {
        let mut repo = create_string_repo();

        // Add and remove an entity
        let id = repo.add("original".to_string()).unwrap();
        repo.remove(id).unwrap();

        // Try to update the removed entity
        let result = repo.update(id, "updated".to_string());
        assert!(matches!(result, Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_get_all_empty() {
        let repo = create_string_repo();

        let all = repo.get_all().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_get_all() {
        let mut repo = create_string_repo();

        // Add multiple entities
        repo.add("entity 1".to_string()).unwrap();
        repo.add("entity 2".to_string()).unwrap();
        repo.add("entity 3".to_string()).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"entity 1".to_string()));
        assert!(all.contains(&"entity 2".to_string()));
        assert!(all.contains(&"entity 3".to_string()));
    }

    #[test]
    fn test_get_all_with_removed() {
        let mut repo = create_string_repo();

        // Add multiple entities
        let id1 = repo.add("entity 1".to_string()).unwrap();
        let id2 = repo.add("entity 2".to_string()).unwrap();
        let id3 = repo.add("entity 3".to_string()).unwrap();

        // Remove one entity
        repo.remove(id2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"entity 1".to_string()));
        assert!(!all.contains(&"entity 2".to_string()));
        assert!(all.contains(&"entity 3".to_string()));
        assert_eq!("entity 1".to_string(), repo.get(id1).unwrap());
        assert_eq!("entity 3".to_string(), repo.get(id3).unwrap());
        // test that getting id2 now gives an error:
        assert!(matches!(repo.get(id2), Err(VecRepositoryError::NotFound)));
    }

    #[test]
    fn test_complex_workflow() {
        let mut repo = create_string_repo();

        // Add entities
        let id1 = repo.add("entity 1".to_string()).unwrap();
        let id2 = repo.add("entity 2".to_string()).unwrap();

        // Update an entity
        repo.update(id1, "updated entity 1".to_string()).unwrap();

        // Remove an entity
        repo.remove(id2).unwrap();

        // Add another entity
        let id3 = repo.add("entity 3".to_string()).unwrap();

        // Check the state
        assert_eq!(repo.get(id1).unwrap(), "updated entity 1");
        assert!(matches!(repo.get(id2), Err(VecRepositoryError::NotFound)));
        assert_eq!(repo.get(id3).unwrap(), "entity 3");

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&"updated entity 1".to_string()));
        assert!(all.contains(&"entity 3".to_string()));
    }

    #[test]
    fn test_with_custom_type() {
        #[derive(Debug, Clone, PartialEq)]
        struct Person {
            name: String,
            age: u32,
        }

        let mut repo: VecRepository<TestId, Person> = VecRepository::new();

        let person1 = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let person2 = Person {
            name: "Bob".to_string(),
            age: 25,
        };

        let id1 = repo.add(person1.clone()).unwrap();
        let id2 = repo.add(person2.clone()).unwrap();

        assert_eq!(repo.get(id1).unwrap(), person1);
        assert_eq!(repo.get(id2).unwrap(), person2);

        let updated_person = Person {
            name: "Alice Smith".to_string(),
            age: 31,
        };

        repo.update(id1, updated_person.clone()).unwrap();
        assert_eq!(repo.get(id1).unwrap(), updated_person);

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_create_assigns_sequential_ids() {
        let mut repo = create_string_repo();

        // Create multiple entities
        let entity1 = repo
            .create(|id| format!("Entity with ID {}", id.value()))
            .unwrap();
        let entity2 = repo
            .create(|id| format!("Entity with ID {}", id.value()))
            .unwrap();
        let entity3 = repo
            .create(|id| format!("Entity with ID {}", id.value()))
            .unwrap();

        // Verify the entities were created with sequential IDs
        assert_eq!(entity1, "Entity with ID 0");
        assert_eq!(entity2, "Entity with ID 1");
        assert_eq!(entity3, "Entity with ID 2");

        // Verify we can retrieve them by ID
        assert_eq!(repo.get(TestId(0)).unwrap(), "Entity with ID 0");
        assert_eq!(repo.get(TestId(1)).unwrap(), "Entity with ID 1");
        assert_eq!(repo.get(TestId(2)).unwrap(), "Entity with ID 2");
    }

    #[test]
    fn test_create_with_custom_type() {
        #[derive(Debug, Clone, PartialEq)]
        struct Person {
            id: TestId,
            name: String,
            age: u32,
        }

        let mut repo: VecRepository<TestId, Person> = VecRepository::new();

        // Create a person using the create method
        let person = repo
            .create(|id| Person {
                id,
                name: "Alice".to_string(),
                age: 30,
            })
            .unwrap();

        // Verify the person was created with the correct ID
        assert_eq!(person.id, TestId(0));
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 30);

        // Create another person
        let person2 = repo
            .create(|id| Person {
                id,
                name: "Bob".to_string(),
                age: 25,
            })
            .unwrap();

        // Verify the second person has the next ID
        assert_eq!(person2.id, TestId(1));

        // Verify we can retrieve them by ID
        let retrieved = repo.get(TestId(0)).unwrap();
        assert_eq!(retrieved.id, TestId(0));
        assert_eq!(retrieved.name, "Alice");

        let retrieved2 = repo.get(TestId(1)).unwrap();
        assert_eq!(retrieved2.id, TestId(1));
        assert_eq!(retrieved2.name, "Bob");
    }

    #[test]
    fn test_create_then_update() {
        let mut repo = create_string_repo();

        // Create an entity
        let entity = repo
            .create(|id| format!("Original entity {}", id.value()))
            .unwrap();
        assert_eq!(entity, "Original entity 0");

        // Update the entity
        let old = repo
            .update(TestId(0), "Updated entity".to_string())
            .unwrap();
        assert_eq!(old, "Original entity 0");

        // Verify the update worked
        assert_eq!(repo.get(TestId(0)).unwrap(), "Updated entity");
    }

    #[test]
    fn test_create_then_remove() {
        let mut repo = create_string_repo();

        // Create an entity
        let entity = repo
            .create(|id| format!("Entity to remove {}", id.value()))
            .unwrap();
        assert_eq!(entity, "Entity to remove 0");

        // Remove the entity
        let removed = repo.remove(TestId(0)).unwrap();
        assert_eq!(removed, "Entity to remove 0");

        // Verify it was removed
        assert!(matches!(
            repo.get(TestId(0)),
            Err(VecRepositoryError::NotFound)
        ));
    }

    #[test]
    fn test_create_with_complex_factory() {
        let mut repo = create_string_repo();

        // Create an entity with a more complex factory function
        let entity = repo
            .create(|id| {
                let base = format!("Complex entity {}", id.value());
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                format!("{} created at {}", base, timestamp)
            })
            .unwrap();

        // Verify it contains the expected base text
        assert!(entity.starts_with("Complex entity 0 created at "));

        // Verify we can retrieve it
        let retrieved = repo.get(TestId(0)).unwrap();
        assert_eq!(retrieved, entity);
    }

    #[test]
    fn test_create_multiple_and_get_all() {
        let mut repo = create_string_repo();

        // Create multiple entities
        repo.create(|_| "Entity A".to_string()).unwrap();
        repo.create(|_| "Entity B".to_string()).unwrap();
        repo.create(|_| "Entity C".to_string()).unwrap();

        // Get all entities
        let all = repo.get_all().unwrap();

        // Verify all entities are present
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"Entity A".to_string()));
        assert!(all.contains(&"Entity B".to_string()));
        assert!(all.contains(&"Entity C".to_string()));
    }
}
