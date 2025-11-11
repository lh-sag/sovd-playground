// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use dashmap::DashMap;

use crate::entities::{Entity, EntityId};

pub struct EntityRepository {
    entities: DashMap<String, Arc<dyn Entity>>,
}

impl EntityRepository {
    pub fn new() -> Self {
        Self {
            entities: DashMap::new(),
        }
    }

    pub fn add_entity(&self, entity: Arc<dyn Entity>) {
        self.entities.insert(entity.id().to_string(), entity);
    }

    pub fn entity_exists(&self, entity_id: &EntityId) -> bool {
        self.entities.contains_key(&entity_id.id)
    }

    pub fn get_entity(&self, id: &str) -> Option<Arc<dyn Entity>> {
        self.entities.get(id).map(|r| r.value().clone())
    }

    pub fn get_entity_by_id(&self, entity_id: &EntityId) -> Option<Arc<dyn Entity>> {
        self.entities.get(&entity_id.id).map(|r| r.value().clone())
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
}

impl Default for EntityRepository {
    fn default() -> Self {
        Self::new()
    }
}
