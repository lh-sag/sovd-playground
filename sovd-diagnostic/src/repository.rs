// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::entities::Entity;

#[derive(Clone)]
pub enum EntityEvent {
    Added { entity_id: String, entity: Arc<dyn Entity> },
    Removed { entity_id: String, entity: Arc<dyn Entity> },
}

pub struct EntityRepository {
    entities: DashMap<String, Arc<dyn Entity>>,
    event_tx: broadcast::Sender<EntityEvent>,
}

impl EntityRepository {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            entities: DashMap::new(),
            event_tx,
        }
    }

    pub fn add_entity(&self, entity: Arc<dyn Entity>) {
        let entity_id = entity.id().to_string();
        self.entities.insert(entity_id.clone(), entity.clone());
        let _ = self.event_tx.send(EntityEvent::Added { entity_id, entity });
    }

    pub fn entity_exists(&self, entity_id: &str) -> bool {
        self.entities.contains_key(entity_id)
    }

    pub fn get_entity(&self, id: &str) -> Option<Arc<dyn Entity>> {
        self.entities.get(id).map(|r| r.value().clone())
    }

    pub fn list_entities(&self) -> Vec<Arc<dyn Entity>> {
        self.entities.iter().map(|r| r.value().clone()).collect()
    }

    pub fn list_components(&self) -> Vec<Arc<dyn Entity>> {
        let components: Vec<Arc<dyn Entity>> = self
            .entities
            .iter()
            .filter(|r| !r.value().id().is_empty())
            .map(|r| r.value().clone())
            .collect();

        tracing::debug!(
            "list_components: total entities={}, filtered components={}, entity ids={:?}",
            self.entities.len(),
            components.len(),
            self.entities.iter().map(|r| r.key().clone()).collect::<Vec<_>>()
        );

        components
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn remove_entity(&self, entity_id: &str) -> Option<Arc<dyn Entity>> {
        self.entities.remove(entity_id).map(|(id, entity)| {
            let _ = self.event_tx.send(EntityEvent::Removed {
                entity_id: id,
                entity: entity.clone(),
            });
            entity
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EntityEvent> {
        self.event_tx.subscribe()
    }
}

impl Default for EntityRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        id: String,
        name: String,
    }

    impl Entity for TestEntity {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn tags(&self) -> &[String] {
            &[]
        }
        fn translation_id(&self) -> Option<&str> {
            None
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_add_and_remove_entity_sends_event() {
        let repo = EntityRepository::new();
        let mut rx = repo.subscribe();

        let entity = Arc::new(TestEntity {
            id: "test".to_string(),
            name: "Test".to_string(),
        });

        repo.add_entity(entity);

        let event = rx.recv().await.expect("Should receive event");
        match event {
            EntityEvent::Added { entity_id, entity } => {
                assert_eq!(entity_id, "test");
                assert_eq!(entity.id(), "test");
                assert_eq!(entity.name(), "Test");
            }
            _ => panic!("Expected Added event"),
        }

        let removed = repo.remove_entity("test").expect("Should remove entity");
        assert_eq!(removed.id(), "test");

        let event = rx.recv().await.expect("Should receive removal event");
        match event {
            EntityEvent::Removed { entity_id, entity } => {
                assert_eq!(entity_id, "test");
                assert_eq!(entity.id(), "test");
                assert_eq!(entity.name(), "Test");
            }
            _ => panic!("Expected Removed event"),
        }
    }
}
