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
pub struct DataEntry {
    pub metadata: ValueMetaData,
    pub value: serde_json::Value,
    pub writable: bool,
}

pub struct MockComponent {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
}

impl MockComponent {
    pub fn new(id: String, name: String, tags: Vec<String>) -> Self {
        Self { id, name, tags }
    }
}

impl Entity for MockComponent {
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
}

pub struct MockDataService {
    data: Arc<RwLock<HashMap<String, DataEntry>>>,
}

impl MockDataService {
    pub fn new(data: HashMap<String, DataEntry>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }
}

#[async_trait]
impl DataService for MockDataService {
    async fn list(
        &self,
        entity: &EntityId,
        categories: Vec<DataCategory>,
        groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError> {
        trace!(entity = %entity, categories = ?categories, groups = ?groups, "List data items");
        let data = self.data.read().await;

        let filtered: Vec<ValueMetaData> = data
            .values()
            .filter(|entry| {
                let category_match = categories.is_empty() || categories.contains(&entry.metadata.category);
                let group_match = groups.is_empty() || entry.metadata.groups.iter().any(|g| groups.contains(g));
                category_match && group_match
            })
            .map(|entry| entry.metadata.clone())
            .collect();

        trace!(entity = %entity, count = filtered.len(), "Filtered data items");
        Ok(filtered)
    }

    async fn read(&self, entity: &EntityId, data_id: &str) -> Result<DataValue, DataError> {
        trace!(entity = %entity, data_id = %data_id, "Read data value");
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
        debug!(entity = %entity, data_id = %data_id, "Write data value");
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
        trace!(entity = %entity, "List categories");
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
        trace!(entity = %entity, category = ?category, "List groups");
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
