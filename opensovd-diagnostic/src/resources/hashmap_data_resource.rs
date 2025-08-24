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

use crate::resources::data::{DataResource, DataError, StringDataItem, StringDataCategory};

/// Internal structure to hold data values with their metadata
#[derive(Debug, Clone)]
struct DataValue {
    /// The actual JSON value
    value: serde_json::Value,
    /// Metadata about this data item
    metadata: StringDataItem,
    /// Whether this data item is read-only
    read_only: bool,
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
        Self {
            data: HashMap::new(),
        }
    }

    /// Creates a HashMapDataResource from a HashMap of JSON values
    ///
    /// This convenience method creates data items with default metadata
    /// (CurrentData category, no groups/tags).
    ///
    /// # Arguments
    /// * `data` - HashMap mapping data IDs to JSON values
    ///
    /// # Returns
    /// A new HashMapDataResource with the provided data
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use serde_json::json;
    /// use opensovd_diagnostic::resources::hashmap_data_resource::HashMapDataResource;
    ///
    /// let mut data = HashMap::new();
    /// data.insert("temperature".to_string(), json!({"value": 85.5, "unit": "celsius"}));
    /// data.insert("pressure".to_string(), json!({"value": 14.7, "unit": "psi"}));
    ///
    /// let resource = HashMapDataResource::from_json_map(data);
    /// ```
    pub fn from_json_map(data: HashMap<String, serde_json::Value>) -> Self {
        let mut resource = Self::new();
        for (id, value) in data {
            let metadata = StringDataItem {
                id: id.clone(),
                name: id.clone(), // Use ID as name by default
                translation_id: None,
                category: StringDataCategory::CurrentData, // Default to current data
                groups: Vec::new(),
                tags: Vec::new(),
            };
            resource.add_data_item(metadata, value, false);
        }
        resource
    }

    /// Adds a new data item to the resource
    ///
    /// # Arguments
    /// * `metadata` - The metadata for the data item
    /// * `value` - The initial JSON value
    /// * `read_only` - Whether this item is read-only
    pub fn add_data_item(&mut self, metadata: StringDataItem, value: serde_json::Value, read_only: bool) {
        let data_value = DataValue {
            value,
            metadata,
            read_only,
        };
        self.data.insert(data_value.metadata.id.clone(), data_value);
    }

    /// Adds a data item with full metadata control
    ///
    /// # Arguments
    /// * `id` - The data item ID
    /// * `name` - Human-readable name
    /// * `category` - The data category
    /// * `groups` - List of groups this item belongs to
    /// * `tags` - List of tags for this item
    /// * `value` - The initial JSON value
    /// * `read_only` - Whether this item is read-only
    pub fn add_data_item_with_metadata(
        &mut self,
        id: String,
        name: String,
        category: StringDataCategory,
        groups: Vec<String>,
        tags: Vec<String>,
        value: serde_json::Value,
        read_only: bool,
    ) {
        let metadata = StringDataItem {
            id: id.clone(),
            name,
            translation_id: None,
            category,
            groups,
            tags,
        };
        self.add_data_item(metadata, value, read_only);
    }

    /// Removes a data item from the resource
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to remove
    ///
    /// # Returns
    /// The removed data item, or None if it didn't exist
    pub fn remove_data_item(&mut self, data_id: &str) -> Option<(StringDataItem, serde_json::Value)> {
        self.data.remove(data_id).map(|dv| (dv.metadata, dv.value))
    }

    /// Gets the number of data items in this resource
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the resource is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl DataResource for HashMapDataResource {
    fn list_data_items(&self, categories: &[StringDataCategory], groups: &[String]) -> Vec<StringDataItem> {
        self.data
            .values()
            .filter(|dv| {
                // Filter by categories (if specified)
                let category_match = categories.is_empty() || categories.contains(&dv.metadata.category);
                
                // Filter by groups (if specified)
                let group_match = groups.is_empty() || 
                    dv.metadata.groups.iter().any(|g| groups.contains(g));
                
                category_match && group_match
            })
            .map(|dv| dv.metadata.clone())
            .collect()
    }

    fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError> {
        self.data
            .get(data_id)
            .map(|dv| dv.value.clone())
            .ok_or_else(|| DataError::not_found(format!("Data item '{}' not found", data_id)))
    }

    fn write_data(&mut self, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        match self.data.get_mut(data_id) {
            Some(data_value) => {
                if data_value.read_only {
                    Err(DataError::read_only(format!("Data item '{}' is read-only", data_id)))
                } else {
                    data_value.value = value;
                    Ok(())
                }
            }
            None => Err(DataError::not_found(format!("Data item '{}' not found", data_id))),
        }
    }

    fn has_data_item(&self, data_id: &str) -> bool {
        self.data.contains_key(data_id)
    }

    fn get_data_item(&self, data_id: &str) -> Option<StringDataItem> {
        self.data.get(data_id).map(|dv| dv.metadata.clone())
    }
}

impl Default for HashMapDataResource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_resource() {
        let resource = HashMapDataResource::new();
        assert!(resource.is_empty());
        assert_eq!(resource.len(), 0);
    }

    #[test]
    fn test_from_json_map() {
        let mut data = HashMap::new();
        data.insert("temperature".to_string(), json!({"value": 85.5, "unit": "celsius"}));
        data.insert("pressure".to_string(), json!({"value": 14.7, "unit": "psi"}));

        let resource = HashMapDataResource::from_json_map(data);
        assert_eq!(resource.len(), 2);
        assert!(resource.has_data_item("temperature"));
        assert!(resource.has_data_item("pressure"));
        
        let temp_value = resource.read_data("temperature").unwrap();
        assert_eq!(temp_value, json!({"value": 85.5, "unit": "celsius"}));
    }

    #[test]
    fn test_add_data_item_with_metadata() {
        let mut resource = HashMapDataResource::new();
        
        resource.add_data_item_with_metadata(
            "engine_rpm".to_string(),
            "Engine RPM".to_string(),
            StringDataCategory::CurrentData,
            vec!["engine".to_string()],
            vec!["rpm".to_string()],
            json!({"value": 2500, "unit": "rpm"}),
            false,
        );

        assert_eq!(resource.len(), 1);
        assert!(resource.has_data_item("engine_rpm"));
        
        let metadata = resource.get_data_item("engine_rpm").unwrap();
        assert_eq!(metadata.name, "Engine RPM");
        assert_eq!(metadata.category, StringDataCategory::CurrentData);
        assert_eq!(metadata.groups, vec!["engine"]);
        assert_eq!(metadata.tags, vec!["rpm"]);
    }

    #[test]
    fn test_read_write_data() {
        let mut resource = HashMapDataResource::new();
        
        resource.add_data_item_with_metadata(
            "test".to_string(),
            "Test Value".to_string(),
            StringDataCategory::CurrentData,
            Vec::new(),
            Vec::new(),
            json!(100),
            false,
        );

        // Test reading
        let value = resource.read_data("test").unwrap();
        assert_eq!(value, json!(100));

        // Test writing
        resource.write_data("test", json!(200)).unwrap();
        let value = resource.read_data("test").unwrap();
        assert_eq!(value, json!(200));

        // Test reading non-existent item
        let result = resource.read_data("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_only_data() {
        let mut resource = HashMapDataResource::new();
        
        resource.add_data_item_with_metadata(
            "readonly".to_string(),
            "Read Only Value".to_string(),
            StringDataCategory::IdentData,
            Vec::new(),
            Vec::new(),
            json!("immutable"),
            true, // read-only
        );

        // Reading should work
        let value = resource.read_data("readonly").unwrap();
        assert_eq!(value, json!("immutable"));

        // Writing should fail
        let result = resource.write_data("readonly", json!("modified"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_data_items_filtering() {
        let mut resource = HashMapDataResource::new();
        
        // Add items with different categories and groups
        resource.add_data_item_with_metadata(
            "id1".to_string(),
            "Ident Data".to_string(),
            StringDataCategory::IdentData,
            vec!["engine".to_string()],
            Vec::new(),
            json!("ident"),
            true,
        );
        
        resource.add_data_item_with_metadata(
            "current1".to_string(),
            "Current Data".to_string(),
            StringDataCategory::CurrentData,
            vec!["engine".to_string()],
            Vec::new(),
            json!(100),
            false,
        );
        
        resource.add_data_item_with_metadata(
            "current2".to_string(),
            "Current Data 2".to_string(),
            StringDataCategory::CurrentData,
            vec!["transmission".to_string()],
            Vec::new(),
            json!(200),
            false,
        );

        // Test no filtering - should get all items
        let all_items = resource.list_data_items(&[], &[]);
        assert_eq!(all_items.len(), 3);

        // Test category filtering
        let ident_items = resource.list_data_items(&[StringDataCategory::IdentData], &[]);
        assert_eq!(ident_items.len(), 1);
        assert_eq!(ident_items[0].id, "id1");

        let current_items = resource.list_data_items(&[StringDataCategory::CurrentData], &[]);
        assert_eq!(current_items.len(), 2);

        // Test group filtering
        let engine_items = resource.list_data_items(&[], &["engine".to_string()]);
        assert_eq!(engine_items.len(), 2);

        let transmission_items = resource.list_data_items(&[], &["transmission".to_string()]);
        assert_eq!(transmission_items.len(), 1);
        assert_eq!(transmission_items[0].id, "current2");

        // Test combined filtering
        let engine_current = resource.list_data_items(&[StringDataCategory::CurrentData], &["engine".to_string()]);
        assert_eq!(engine_current.len(), 1);
        assert_eq!(engine_current[0].id, "current1");
    }

    #[test]
    fn test_remove_data_item() {
        let mut resource = HashMapDataResource::new();
        
        resource.add_data_item_with_metadata(
            "test".to_string(),
            "Test".to_string(),
            StringDataCategory::CurrentData,
            Vec::new(),
            Vec::new(),
            json!(42),
            false,
        );

        assert!(resource.has_data_item("test"));
        
        let removed = resource.remove_data_item("test");
        assert!(removed.is_some());
        let (metadata, value) = removed.unwrap();
        assert_eq!(metadata.id, "test");
        assert_eq!(value, json!(42));
        
        assert!(!resource.has_data_item("test"));
        assert!(resource.is_empty());
    }
}