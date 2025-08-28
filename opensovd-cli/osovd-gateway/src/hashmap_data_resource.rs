// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0
//

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use opensovd_diagnostic::resources::data::{DataError, DataItem, DataResource};

/// Structure to hold data values with their metadata
#[derive(Debug, Clone)]
pub struct DataValue {
    /// The actual JSON value
    pub value: serde_json::Value,
    /// Metadata about this data item
    pub metadata: DataItem,
    /// Whether this data item is read-only
    pub read_only: bool,
}

/// ISO 17978-3 compliant HashMap-based data resource implementation
///
/// This implementation stores data as JSON values with associated metadata
/// including categories, groups, and other ISO-required attributes.
#[derive(Debug)]
pub struct HashMapDataResource {
    /// Internal storage mapping data IDs to values and metadata
    data: HashMap<String, DataValue>,
}

impl HashMapDataResource {
    /// Creates a new empty HashMapDataResource
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Inserts a data value directly into the resource
    ///
    /// # Arguments
    /// * `data_value` - The DataValue to insert
    pub fn insert(&mut self, data_value: DataValue) {
        self.data.insert(data_value.metadata.id.clone(), data_value);
    }
}

impl DataResource for HashMapDataResource {
    fn list_data_items<'a>(&'a self, categories: &'a [String], groups: &'a [String]) 
        -> Pin<Box<dyn Future<Output = Vec<DataItem>> + Send + 'a>> 
    {
        Box::pin(async move {
            self.data
                .values()
                .filter(|dv| {
                    // Filter by categories (if specified)
                    let category_match = categories.is_empty() || categories.contains(&dv.metadata.category);

                    // Filter by groups (if specified)
                    let group_match = groups.is_empty() || dv.metadata.groups.iter().any(|g| groups.contains(g));

                    category_match && group_match
                })
                .map(|dv| DataItem {
                    id: dv.metadata.id.clone(),
                    name: dv.metadata.name.clone(),
                    category: dv.metadata.category.clone(),
                    groups: dv.metadata.groups.clone(),
                    tags: dv.metadata.tags.clone(),
                })
                .collect()
        })
    }

    fn read_data<'a>(&'a self, data_id: &'a str) 
        -> Pin<Box<dyn Future<Output = Result<serde_json::Value, DataError>> + Send + 'a>> 
    {
        Box::pin(async move {
            self.data
                .get(data_id)
                .map(|dv| dv.value.clone())
                .ok_or_else(|| DataError::DataNotFound(data_id.to_string()))
        })
    }

    fn write_data<'a>(&'a mut self, data_id: &'a str, value: serde_json::Value) 
        -> Pin<Box<dyn Future<Output = Result<(), DataError>> + Send + 'a>> 
    {
        Box::pin(async move {
            match self.data.get_mut(data_id) {
                Some(data_value) => {
                    if data_value.read_only {
                        Err(DataError::ReadOnly(data_id.to_string()))
                    } else {
                        data_value.value = value;
                        Ok(())
                    }
                }
                None => Err(DataError::DataNotFound(data_id.to_string())),
            }
        })
    }

    fn has_data_item<'a>(&'a self, data_id: &'a str) 
        -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> 
    {
        Box::pin(async move {
            self.data.contains_key(data_id)
        })
    }

    fn get_data_item<'a>(&'a self, data_id: &'a str) 
        -> Pin<Box<dyn Future<Output = Option<DataItem>> + Send + 'a>> 
    {
        Box::pin(async move {
            self.data.get(data_id).map(|dv| DataItem {
                id: dv.metadata.id.clone(),
                name: dv.metadata.name.clone(),
                category: dv.metadata.category.clone(),
                groups: dv.metadata.groups.clone(),
                tags: dv.metadata.tags.clone(),
            })
        })
    }
}

impl Default for HashMapDataResource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_new_resource() {
        let _resource = HashMapDataResource::new();
        // Test that we can create a new resource without panicking
    }

    #[tokio::test]
    async fn test_insert_data_value() {
        let mut resource = HashMapDataResource::new();

        let metadata = DataItem {
            id: "engine_rpm".to_string(),
            name: "Engine RPM".to_string(),
            category: "currentData".to_string(),
            groups: vec!["engine".to_string()],
            tags: vec!["rpm".to_string()],
        };
        let data_value = DataValue {
            value: json!({"value": 2500, "unit": "rpm"}),
            metadata,
            read_only: false,
        };
        resource.insert(data_value);

        // Resource has been created successfully
        assert!(resource.has_data_item("engine_rpm").await);

        let metadata = resource.get_data_item("engine_rpm").await.unwrap();
        assert_eq!(metadata.name, "Engine RPM");
        assert_eq!(metadata.category, "currentData");
        assert_eq!(metadata.groups, vec!["engine"]);
        assert_eq!(metadata.tags, vec!["rpm"]);
    }

    #[tokio::test]
    async fn test_read_write_data() {
        let mut resource = HashMapDataResource::new();

        let metadata = DataItem {
            id: "test".to_string(),
            name: "Test Value".to_string(),
            category: "currentData".to_string(),
            groups: Vec::new(),
            tags: Vec::new(),
        };
        let data_value = DataValue {
            value: json!(100),
            metadata,
            read_only: false,
        };
        resource.insert(data_value);

        // Test reading
        let value = resource.read_data("test").await.unwrap();
        assert_eq!(value, json!(100));

        // Test writing
        resource.write_data("test", json!(200)).await.unwrap();
        let value = resource.read_data("test").await.unwrap();
        assert_eq!(value, json!(200));

        // Test reading non-existent item
        let result = resource.read_data("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_only_data() {
        let mut resource = HashMapDataResource::new();

        let metadata = DataItem {
            id: "readonly".to_string(),
            name: "Read Only Value".to_string(),
            category: "identData".to_string(),
            groups: Vec::new(),
            tags: Vec::new(),
        };
        let data_value = DataValue {
            value: json!("immutable"),
            metadata,
            read_only: true, // read-only
        };
        resource.insert(data_value);

        // Reading should work
        let value = resource.read_data("readonly").await.unwrap();
        assert_eq!(value, json!("immutable"));

        // Writing should fail
        let result = resource.write_data("readonly", json!("modified")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_data_items_filtering() {
        let mut resource = HashMapDataResource::new();

        // Add items with different categories and groups
        let metadata = DataItem {
            id: "id1".to_string(),
            name: "Ident Data".to_string(),
            category: "identData".to_string(),
            groups: vec!["engine".to_string()],
            tags: Vec::new(),
        };
        let data_value = DataValue {
            value: json!("ident"),
            metadata,
            read_only: true,
        };
        resource.insert(data_value);

        let metadata = DataItem {
            id: "current1".to_string(),
            name: "Current Data".to_string(),
            category: "currentData".to_string(),
            groups: vec!["engine".to_string()],
            tags: Vec::new(),
        };
        let data_value = DataValue {
            value: json!(100),
            metadata,
            read_only: false,
        };
        resource.insert(data_value);

        let metadata = DataItem {
            id: "current2".to_string(),
            name: "Current Data 2".to_string(),
            category: "currentData".to_string(),
            groups: vec!["transmission".to_string()],
            tags: Vec::new(),
        };
        let data_value = DataValue {
            value: json!(200),
            metadata,
            read_only: false,
        };
        resource.insert(data_value);

        // Test no filtering - should get all items
        let all_items = resource.list_data_items(&[], &[]).await;
        assert_eq!(all_items.len(), 3);

        // Test category filtering
        let ident_items = resource.list_data_items(&["identData".to_string()], &[]).await;
        assert_eq!(ident_items.len(), 1);
        assert_eq!(ident_items[0].id, "id1");

        let current_items = resource.list_data_items(&["currentData".to_string()], &[]).await;
        assert_eq!(current_items.len(), 2);

        // Test group filtering
        let engine_items = resource.list_data_items(&[], &["engine".to_string()]).await;
        assert_eq!(engine_items.len(), 2);

        let transmission_items = resource.list_data_items(&[], &["transmission".to_string()]).await;
        assert_eq!(transmission_items.len(), 1);
        assert_eq!(transmission_items[0].id, "current2");

        // Test combined filtering
        let engine_current = resource.list_data_items(&["currentData".to_string()], &["engine".to_string()]).await;
        assert_eq!(engine_current.len(), 1);
        assert_eq!(engine_current[0].id, "current1");
    }
}
