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

use crate::middleware::{Next, ResourceMiddleware};
use crate::resources::{DataError, DataItem, HealthStatus, OperationResult, Resource, ResourceOperation};

/// A Component represents a diagnostic entity with an id, name, resource, and middleware chain
pub struct Component {
    id: String,
    name: String,
    resource: Box<dyn Resource>,
    middleware: Vec<Arc<dyn ResourceMiddleware>>,
}

impl Component {
    /// Creates a new component builder
    pub fn builder(id: String, name: String) -> ComponentBuilder {
        ComponentBuilder {
            id,
            name,
            resource: None,
            middleware: Vec::new(),
        }
    }
    
    /// Returns the component's id
    pub fn id(&self) -> &String {
        &self.id
    }
    
    /// Returns the component's name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Execute an operation through the middleware chain
    async fn execute(&self, operation: ResourceOperation) -> Result<OperationResult, DataError> {
        let next = Next::new(&self.middleware[..], self.resource.as_ref());
        next.run(operation).await
    }
    
    /// List all data items with optional filtering
    pub async fn list_data_items(&self, categories: &[String], groups: &[String]) -> Result<Vec<DataItem>, DataError> {
        let op = ResourceOperation::ListData {
            categories: categories.to_vec(),
            groups: groups.to_vec(),
        };
        
        match self.execute(op).await? {
            OperationResult::DataItems(items) => Ok(items),
            _ => Err(DataError::InternalError("Unexpected result".into())),
        }
    }
    
    /// Read a specific data value
    pub async fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError> {
        let op = ResourceOperation::ReadData {
            data_id: data_id.to_string(),
        };
        
        match self.execute(op).await? {
            OperationResult::DataValue(value) => Ok(value),
            _ => Err(DataError::InternalError("Unexpected result".into())),
        }
    }
    
    /// Write a specific data value
    pub async fn write_data(&self, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        let op = ResourceOperation::WriteData {
            data_id: data_id.to_string(),
            value,
        };
        
        match self.execute(op).await? {
            OperationResult::Success => Ok(()),
            _ => Err(DataError::InternalError("Unexpected result".into())),
        }
    }
    
    /// Check if a data item exists
    pub async fn has_data_item(&self, data_id: &str) -> bool {
        let op = ResourceOperation::HasDataItem {
            data_id: data_id.to_string(),
        };
        
        match self.execute(op).await {
            Ok(OperationResult::Bool(exists)) => exists,
            _ => false,
        }
    }
    
    /// Get metadata for a specific data item
    pub async fn get_data_item(&self, data_id: &str) -> Option<DataItem> {
        let op = ResourceOperation::GetDataItem {
            data_id: data_id.to_string(),
        };
        
        match self.execute(op).await {
            Ok(OperationResult::DataItem(item)) => item,
            _ => None,
        }
    }
    
    /// Health check for the component
    pub async fn health_check(&self) -> HealthStatus {
        let op = ResourceOperation::HealthCheck;
        
        match self.execute(op).await {
            Ok(OperationResult::Health(status)) => status,
            _ => HealthStatus::Unhealthy("Health check failed".into()),
        }
    }
}

/// Builder for Component
pub struct ComponentBuilder {
    id: String,
    name: String,
    resource: Option<Box<dyn Resource>>,
    middleware: Vec<Arc<dyn ResourceMiddleware>>,
}

impl ComponentBuilder {
    /// Set the resource for this component
    pub fn with_resource(mut self, resource: Box<dyn Resource>) -> Self {
        self.resource = Some(resource);
        self
    }
    
    /// Add middleware to the chain
    pub fn with_middleware(mut self, middleware: Arc<dyn ResourceMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }
    
    /// Build the component
    pub fn build(self) -> Component {
        Component {
            id: self.id,
            name: self.name,
            resource: self.resource.expect("Resource is required"),
            middleware: self.middleware,
        }
    }
}