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

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::entities::Component;

pub mod resolvers;

/// Trait for component resolvers that can dynamically discover components
#[async_trait]
pub trait ComponentResolver: Send + Sync + 'static {
    /// Try to resolve a component by ID
    async fn resolve(&self, id: &str) -> Option<Component>;
    
    /// Priority of this resolver (higher = tried first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Registry for managing and discovering components
pub struct ComponentRegistry {
    /// Static components cache
    static_components: Arc<RwLock<HashMap<String, Arc<Component>>>>,
    /// Dynamic resolvers
    resolvers: Vec<Arc<dyn ComponentResolver>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            static_components: Arc::new(RwLock::new(HashMap::new())),
            resolvers: Vec::new(),
        }
    }
    
    /// Create a builder for the registry
    pub fn builder() -> ComponentRegistryBuilder {
        ComponentRegistryBuilder::new()
    }
    
    /// Get a component by ID, trying static components first, then resolvers
    pub async fn get_component(&self, id: &str) -> Option<Arc<Component>> {
        // First check static components
        {
            let components = self.static_components.read().await;
            if let Some(component) = components.get(id) {
                return Some(component.clone());
            }
        }
        
        // Try dynamic resolvers in priority order
        for resolver in &self.resolvers {
            if let Some(component) = resolver.resolve(id).await {
                // Cache it for future use
                let arc_component: Arc<Component> = Arc::new(component);
                let mut components = self.static_components.write().await;
                components.insert(id.to_string(), arc_component.clone());
                return Some(arc_component);
            }
        }
        
        None
    }
    
    /// List all known component IDs
    pub async fn list_component_ids(&self) -> Vec<String> {
        let components = self.static_components.read().await;
        components.keys().cloned().collect()
    }
    
    /// Add a static component to the registry
    pub async fn add_component(&self, component: Component) {
        let id = component.id().clone();
        let mut components = self.static_components.write().await;
        components.insert(id, Arc::new(component));
    }
    
    /// Remove a component from the static cache
    pub async fn remove_component(&self, id: &str) -> Option<Arc<Component>> {
        let mut components = self.static_components.write().await;
        components.remove(id)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for ComponentRegistry
pub struct ComponentRegistryBuilder {
    static_components: Vec<Component>,
    resolvers: Vec<Arc<dyn ComponentResolver>>,
}

impl ComponentRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            static_components: Vec::new(),
            resolvers: Vec::new(),
        }
    }
    
    /// Add static components
    pub fn with_static_components(mut self, components: Vec<Component>) -> Self {
        self.static_components.extend(components);
        self
    }
    
    /// Add a single static component
    pub fn with_component(mut self, component: Component) -> Self {
        self.static_components.push(component);
        self
    }
    
    /// Add a resolver
    pub fn with_resolver(mut self, resolver: impl ComponentResolver + 'static) -> Self {
        self.resolvers.push(Arc::new(resolver));
        self
    }
    
    /// Build the registry
    pub fn build(mut self) -> ComponentRegistry {
        // Sort resolvers by priority (descending)
        self.resolvers.sort_by_key(|r| -r.priority());
        
        let registry = ComponentRegistry {
            static_components: Arc::new(RwLock::new(HashMap::new())),
            resolvers: self.resolvers,
        };
        
        // Add static components
        let runtime = tokio::runtime::Handle::current();
        for component in self.static_components {
            let id = component.id().clone();
            runtime.block_on(async {
                let mut components = registry.static_components.write().await;
                components.insert(id, Arc::new(component));
            });
        }
        
        registry
    }
}