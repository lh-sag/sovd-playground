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

use std::any::Any;

/// Pure POD resource container that holds a single data resource
#[derive(Debug)]
pub struct Resource {
    pub data_resource: Option<Box<dyn Any + Send + Sync>>,
}

impl Resource {
    /// Creates a new empty Resource container
    pub fn new() -> Self {
        Self { data_resource: None }
    }

    /// Creates a new Resource container with the given data resource
    ///
    /// This is a convenience method for creating a Resource that already has
    /// a data resource, avoiding the need to create the resource first and
    /// then set the data separately.
    ///
    /// # Arguments
    ///
    /// * `data_resource` - Any type implementing the Data trait to be stored as the resource's data
    ///
    /// # Returns
    ///
    /// A new Resource instance with the specified data resource
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opensovd_diagnostic::resources::{Resource, HashMapData};
    /// # use opensovd_diagnostic::resources::data::Data;
    /// let mut engine_data: HashMapData<String> = HashMapData::new();
    /// engine_data.write("rpm", "2500".to_string());
    ///
    /// let resource = Resource::with_data_resource(engine_data);
    ///
    /// assert!(resource.has_data_resource());
    /// ```
    pub fn with_data_resource<T: crate::resources::data::Data + 'static>(data_resource: T) -> Self {
        Self {
            data_resource: Some(Box::new(data_resource)),
        }
    }

    /// Get the data resource by type
    pub fn get_data_resource<T: crate::resources::data::Data + 'static>(&self) -> Option<&T> {
        self.data_resource
            .as_ref()
            .and_then(|any_box| any_box.downcast_ref::<T>())
    }

    /// Get mutable access to the data resource by type
    pub fn get_data_resource_mut<T: crate::resources::data::Data + 'static>(&mut self) -> Option<&mut T> {
        self.data_resource
            .as_mut()
            .and_then(|any_box| any_box.downcast_mut::<T>())
    }

    /// Check if this container has a data resource
    pub fn has_data_resource(&self) -> bool {
        self.data_resource.is_some()
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}

pub mod data;
pub mod hashmap_data;

pub use data::Data;
pub use hashmap_data::HashMapData;
