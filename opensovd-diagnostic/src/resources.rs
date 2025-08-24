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

use crate::resources::data::DataResource;

/// Resource container for diagnostic resources
/// 
/// This container holds diagnostic resources as defined by ISO 17978-3,
/// including data resources, fault resources, operations, etc.
pub struct Resource {
    /// ISO-compliant data resource
    pub data: Option<Box<dyn DataResource>>,
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("has_data_resource", &self.data.is_some())
            .finish()
    }
}

impl Resource {
    /// Creates a new empty Resource container
    pub fn new() -> Self {
        Self { 
            data: None,
        }
    }

    /// Creates a new Resource container with the given ISO-compliant data resource
    ///
    /// # Arguments
    ///
    /// * `data_resource` - A DataResource implementation to be stored
    ///
    /// # Returns
    ///
    /// A new Resource instance with the specified data resource
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opensovd_diagnostic::resources::{Resource, HashMapDataResource};
    /// # use serde_json::json;
    /// # use std::collections::HashMap;
    /// let mut data_map = HashMap::new();
    /// data_map.insert("temperature".to_string(), json!({"value": 85.5, "unit": "celsius"}));
    /// 
    /// let data_resource = HashMapDataResource::from_json_map(data_map);
    /// let resource = Resource::with_data_resource(data_resource);
    ///
    /// assert!(resource.has_data_resource());
    /// ```
    pub fn with_data_resource<T: DataResource>(data_resource: T) -> Self {
        Self {
            data: Some(Box::new(data_resource)),
        }
    }


    /// Get the data resource
    pub fn get_data_resource(&self) -> Option<&dyn DataResource> {
        self.data.as_ref().map(|r| r.as_ref())
    }

    /// Get mutable access to the data resource
    pub fn get_data_resource_mut(&mut self) -> Option<&mut dyn DataResource> {
        self.data.as_mut().map(|r| r.as_mut())
    }

    /// Check if this container has a data resource
    pub fn has_data_resource(&self) -> bool {
        self.data.is_some()
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}

pub mod data;
pub mod hashmap_data_resource;

pub use data::{DataItem, DataCategory, DataError, StandardDataCategory, StringDataCategory, StandardDataItem, StringDataItem};
pub use hashmap_data_resource::HashMapDataResource;
