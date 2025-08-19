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
use std::sync::Arc;

use crate::entities::Component;

/// Builder for constructing a Diagnostic system
///
/// The DiagnosticBuilder provides a simple API for creating a Diagnostic system
/// by adding pre-built components.
///
/// # Examples
///
/// ```rust
/// # use opensovd_diagnostic::diagnostic::Diagnostic;
/// # use opensovd_diagnostic::entities::Component;
/// let component = Component::new("engine".into(), "Engine ECU".into());
/// let diagnostic = Diagnostic::builder()
///     .add_component(component)
///     .build();
/// ```
pub struct DiagnosticBuilder {
    components: HashMap<String, Component>,
}

impl DiagnosticBuilder {
    /// Creates a new DiagnosticBuilder
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Adds a component to the builder
    ///
    /// # Arguments
    ///
    /// * `component` - A Component instance to be added to the diagnostic system
    ///
    /// # Returns
    ///
    /// The DiagnosticBuilder for further chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use opensovd_diagnostic::diagnostic::Diagnostic;
    /// # use opensovd_diagnostic::entities::Component;
    /// let component = Component::new("engine".into(), "Engine ECU".into());
    /// let diagnostic = Diagnostic::builder()
    ///     .add_component(component)
    ///     .build();
    /// ```
    pub fn add_component(mut self, component: Component) -> Self {
        self.components.insert(component.id().clone(), component);
        self
    }

    /// Builds the immutable Diagnostic system
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            components: Arc::new(self.components),
        }
    }
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The main diagnostic system containing components (immutable after creation)
pub struct Diagnostic {
    components: Arc<HashMap<String, Component>>,
}

impl Diagnostic {
    /// Creates a new empty diagnostic system
    pub fn new() -> Self {
        Self {
            components: Arc::new(HashMap::new()),
        }
    }

    /// Creates a new DiagnosticBuilder
    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::new()
    }

    /// Retrieves a component by id
    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    /// Returns a list of all component IDs
    pub fn component_ids(&self) -> impl Iterator<Item = &String> {
        self.components.keys()
    }

    /// Returns an iterator over all components
    pub fn components(&self) -> impl Iterator<Item = &Component> {
        self.components.values()
    }

    /// Returns an iterator over all component entries (id, component pairs)
    pub fn component_entries(&self) -> impl Iterator<Item = (&String, &Component)> {
        self.components.iter()
    }

    /// Returns the number of components in the system
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Checks if a component with the given id exists
    pub fn has_component(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }
}

impl Default for Diagnostic {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Diagnostic {
    fn clone(&self) -> Self {
        Self {
            components: Arc::clone(&self.components),
        }
    }
}
