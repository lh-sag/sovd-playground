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

use crate::resources::Resource;

/// A Component represents a diagnostic entity with an id, name, and resource
#[derive(Debug)]
pub struct Component {
    id: String,
    name: String,
    resource: Resource,
}

impl Component {
    /// Creates a new component with the given id and name
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            resource: Resource::new(),
        }
    }

    /// Creates a new component with the given id, name, and resource
    ///
    /// This is a convenience method for creating a component that already has
    /// a configured resource, avoiding the need to create the component first and
    /// then set the resource separately.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the component
    /// * `name` - Human-readable name for the component
    /// * `resource` - A Resource instance to be used by this component
    ///
    /// # Returns
    ///
    /// A new Component instance with the specified id, name, and resource
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opensovd_diagnostic::entities::Component;
    /// # use opensovd_diagnostic::resources::{Resource, HashMapDataResource};
    /// # use opensovd_models::data::StringDataCategory;
    /// # use serde_json::json;
    /// let mut engine_data = HashMapDataResource::new();
    /// engine_data.add_data_item_with_metadata(
    ///     "rpm".to_string(),
    ///     "Engine RPM".to_string(),
    ///     StringDataCategory::CurrentData,
    ///     vec!["engine".to_string()],
    ///     vec!["rpm".to_string()],
    ///     json!({"value": 2500, "unit": "rpm"}),
    ///     false
    /// );
    ///
    /// let component = Component::new_with_resources(
    ///     "engine".to_string(),
    ///     "Engine ECU".to_string(),
    ///     Resource::with_data_resource(engine_data)
    /// );
    ///
    /// assert_eq!(component.id(), "engine");
    /// assert_eq!(component.name(), "Engine ECU");
    /// assert!(component.resource().has_data_resource());
    /// ```
    pub fn new_with_resources(id: String, name: String, resource: Resource) -> Self {
        Self { id, name, resource }
    }
}

impl Component {
    /// Returns the component's id
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Returns the component's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the component's resource
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    /// Gets mutable access to the component's resource
    pub fn resource_mut(&mut self) -> &mut Resource {
        &mut self.resource
    }
}
