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
use crate::resources::data::DataResource;

#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum BuilderError {
    #[display("Component with id '{}' already exists", _0)]
    DuplicateComponent(#[error(ignore)] String),
}


pub struct DiagnosticBuilder {
    components: HashMap<String, Component>,
    next_id: usize,
}

impl DiagnosticBuilder {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            next_id: 1,
        }
    }


    pub fn component(mut self, name: impl Into<String>) -> ComponentBuilder {
        let id = self.next_id.to_string();
        self.next_id += 1;
        ComponentBuilder {
            parent: self,
            id,
            name: name.into(),
            data_resource: None,
        }
    }

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

pub struct Diagnostic {
    components: Arc<HashMap<String, Component>>,
}

impl Diagnostic {
    pub fn new() -> Self {
        Self {
            components: Arc::new(HashMap::new()),
        }
    }

    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::new()
    }

    pub fn get_component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn component_ids(&self) -> impl Iterator<Item = &String> {
        self.components.keys()
    }

    pub fn components(&self) -> impl Iterator<Item = &Component> {
        self.components.values()
    }

    pub fn component_entries(&self) -> impl Iterator<Item = (&String, &Component)> {
        self.components.iter()
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

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

pub struct ComponentBuilder {
    parent: DiagnosticBuilder,
    id: String,
    name: String,
    data_resource: Option<Box<dyn DataResource>>,
}

impl ComponentBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn data_resource<T: DataResource + 'static>(mut self, data_resource: T) -> Self {
        self.data_resource = Some(Box::new(data_resource));
        self
    }

    pub fn add(mut self) -> Result<DiagnosticBuilder, BuilderError> {
        if self.parent.components.contains_key(&self.id) {
            return Err(BuilderError::DuplicateComponent(self.id.clone()));
        }
        
        let mut component = Component::new(self.id.clone(), self.name);
        if let Some(data) = self.data_resource {
            component.data_resource(data);
        }
        
        self.parent.components.insert(self.id, component);
        Ok(self.parent)
    }
}

