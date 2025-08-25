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

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::resources::data::DataResource;

/// Generic lockable wrapper for resources with optional metrics
pub struct Lockable<T> {
    inner: Arc<RwLock<T>>,
    #[cfg(feature = "metrics")]
    resource_type: &'static str,
}

impl<T> Lockable<T> {
    pub fn new(value: T) -> Self {
        Self::new_with_type(value, "unknown")
    }
    
    pub fn new_with_type(value: T, #[allow(unused_variables)] resource_type: &'static str) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            #[cfg(feature = "metrics")]
            resource_type,
        }
    }
    
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
        #[cfg(feature = "metrics")]
        {
            let _timer = crate::metrics::record_lock_acquisition(self.resource_type, "read");
        }
        
        self.inner.read().await
    }
    
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
        #[cfg(feature = "metrics")]
        {
            let _timer = crate::metrics::record_lock_acquisition(self.resource_type, "write");
        }
        
        self.inner.write().await
    }
}

impl<T> Clone for Lockable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            #[cfg(feature = "metrics")]
            resource_type: self.resource_type,
        }
    }
}

/// Resource container for diagnostic resources
///
/// This container holds diagnostic resources as defined by ISO 17978-3,
/// including data resources, fault resources, operations, etc.
pub struct Resource {
    /// ISO-compliant data resource with fine-grained locking
    pub data: Option<Lockable<Box<dyn DataResource>>>,
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
        Self { data: None }
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
    /// # use opensovd_diagnostic::resources::Resource;
    /// # use opensovd_diagnostic::resources::data::{DataResource, DataItem, DataError};
    /// # struct DummyDataResource;
    /// # impl DataResource for DummyDataResource {
    /// #     fn list_data_items(&self, _: &[String], _: &[String]) -> Vec<DataItem> { vec![] }
    /// #     fn read_data(&self, _: &str) -> Result<serde_json::Value, DataError> { todo!() }
    /// #     fn write_data(&mut self, _: &str, _: serde_json::Value) -> Result<(), DataError> { todo!() }
    /// #     fn has_data_item(&self, _: &str) -> bool { false }
    /// #     fn get_data_item(&self, _: &str) -> Option<DataItem> { None }
    /// # }
    /// let data_resource = DummyDataResource;
    /// let resource = Resource::with_data_resource(data_resource);
    ///
    /// assert!(resource.has_data_resource());
    /// ```
    pub fn with_data_resource(data_resource: Box<dyn DataResource>) -> Self {
        Self {
            data: Some(Lockable::new_with_type(data_resource, "data")),
        }
    }

    /// Get the lockable data resource
    pub fn get_data_resource(&self) -> Option<&Lockable<Box<dyn DataResource>>> {
        self.data.as_ref()
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

pub use data::{DataError, DataItem};
