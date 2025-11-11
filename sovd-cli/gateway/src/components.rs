// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sovd_diagnostic::{
    Entity, EntityId,
    data::{DataCategory, DataCategoryInformation, DataError, DataService, DataValue, ValueGroup, ValueMetaData},
};
use tokio::sync::RwLock;
use tracing::{debug, trace};

#[derive(Clone, Debug)]
struct DataEntry {
    metadata: ValueMetaData,
    value: serde_json::Value,
    writable: bool,
}

/// A static component with data from configuration
#[derive(Clone, derive_more::Debug)]
pub struct StaticComponent {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,

    // Component-specific config passed from TOML
    #[debug(skip)]
    #[allow(dead_code)] // Used for get_config_* methods
    pub config: Option<toml::Value>,

    // Static data storage
    #[debug(skip)]
    data: Arc<RwLock<HashMap<String, DataEntry>>>,
}

impl StaticComponent {
    pub fn new(id: String, name: String, tags: Vec<String>, config: Option<toml::Value>) -> Self {
        Self {
            id,
            name,
            tags,
            config,
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a data item (convenience method for config loading)
    pub fn add_data_item(&mut self, metadata: ValueMetaData, value: serde_json::Value, writable: bool) {
        // During initialization, we have exclusive access via &mut self
        // So we can use Arc::get_mut to get direct access without locking
        if let Some(data) = Arc::get_mut(&mut self.data) {
            debug!(
                component_id = %self.id,
                data_id = %metadata.id,
                data_name = %metadata.name,
                category = ?metadata.category,
                groups = ?metadata.groups,
                "Add data item to component"
            );
            data.get_mut().insert(
                metadata.id.clone(),
                DataEntry {
                    metadata,
                    value,
                    writable,
                },
            );
            debug!(component_id = %self.id, count = data.get_mut().len(), "Component data items updated");
        } else {
            // This shouldn't happen during initialization, but handle it gracefully
            panic!("Cannot add data item: Arc is shared during initialization");
        }
    }
}

/// Implement Entity trait for `StaticComponent`
impl Entity for StaticComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn tags(&self) -> &[String] {
        &self.tags
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }

    fn data_service(&self) -> Option<&dyn DataService> {
        Some(self)
    }
}

/// `StaticComponent` implements `DataService` directly
#[async_trait]
impl DataService for StaticComponent {
    async fn list(
        &self,
        entity: &EntityId,
        categories: Vec<DataCategory>,
        groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError> {
        debug!(
            entity_id = %entity.id,
            component_id = %self.id,
            categories = ?categories,
            groups = ?groups,
            "DataService list called"
        );

        // Only respond to requests for this component
        if entity.id != self.id {
            debug!(entity_id = %entity.id, component_id = %self.id, "Entity ID mismatch");
            return Ok(Vec::new());
        }

        let data = self.data.read().await;
        debug!(component_id = %self.id, total_count = data.len(), "Component data items loaded");

        let items: Vec<ValueMetaData> = data
            .values()
            .filter(|entry| {
                let cat_match = categories.is_empty() || categories.contains(&entry.metadata.category);
                let group_match = groups.is_empty() || entry.metadata.groups.iter().any(|g| groups.contains(g));
                let matches = cat_match && group_match;
                trace!(
                    data_id = %entry.metadata.id,
                    cat_match,
                    group_match,
                    matches,
                    "Filter data item"
                );
                matches
            })
            .map(|entry| entry.metadata.clone())
            .collect();
        drop(data);

        debug!(component_id = %self.id, filtered_count = items.len(), "Return filtered items");
        Ok(items)
    }

    async fn read(&self, entity: &EntityId, data_id: &str) -> Result<DataValue, DataError> {
        if entity.id != self.id {
            return Err(DataError::not_found("Wrong component"));
        }

        let data = self.data.read().await;
        data.get(data_id)
            .map(|entry| DataValue {
                id: data_id.to_string(),
                value: entry.value.clone(),
                errors: vec![],
            })
            .ok_or_else(|| DataError::not_found(data_id))
    }

    async fn write(&self, entity: &EntityId, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        if entity.id != self.id {
            return Err(DataError::not_found("Wrong component"));
        }

        let mut data = self.data.write().await;
        match data.get_mut(data_id) {
            Some(entry) if entry.writable => {
                entry.value = value;
                Ok(())
            }
            Some(_) => Err(DataError::read_only(data_id)),
            None => Err(DataError::not_found(data_id)),
        }
    }

    async fn list_categories(&self, entity: &EntityId) -> Result<Vec<DataCategoryInformation>, DataError> {
        if entity.id != self.id {
            return Ok(Vec::new());
        }

        let data = self.data.read().await;
        let mut categories = std::collections::HashSet::new();

        for entry in data.values() {
            categories.insert(entry.metadata.category.clone());
        }
        drop(data);

        Ok(categories
            .into_iter()
            .map(|cat| DataCategoryInformation {
                item: cat,
                category_translation_id: None,
            })
            .collect())
    }

    async fn list_groups(
        &self,
        entity: &EntityId,
        category: Option<DataCategory>,
    ) -> Result<Vec<ValueGroup>, DataError> {
        if entity.id != self.id {
            return Ok(Vec::new());
        }

        let data = self.data.read().await;
        let mut groups = std::collections::HashSet::new();

        for entry in data.values() {
            if let Some(ref cat) = category
                && entry.metadata.category != *cat
            {
                continue;
            }
            for group in &entry.metadata.groups {
                groups.insert((group.clone(), entry.metadata.category.clone()));
            }
        }
        drop(data);

        Ok(groups
            .into_iter()
            .map(|(group, category)| ValueGroup {
                id: group.clone(),
                category,
                category_translation_id: None,
                group: Some(group),
                group_translation_id: None,
            })
            .collect())
    }
}
