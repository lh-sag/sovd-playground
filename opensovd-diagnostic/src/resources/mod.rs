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
use serde::{Deserialize, Serialize};

pub mod data;
pub mod local;
pub mod remote;

pub use data::{DataError, DataItem, DataResource};
pub use local::LocalResource;
pub use remote::RemoteResource;

/// Health status of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Core trait for all resource implementations
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    /// List all data items with optional filtering
    async fn list_data_items(&self, categories: &[String], groups: &[String]) -> Result<Vec<DataItem>, DataError>;
    
    /// Read a specific data value
    async fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError>;
    
    /// Write a specific data value
    async fn write_data(&self, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;
    
    /// Check if a data item exists
    async fn has_data_item(&self, data_id: &str) -> bool;
    
    /// Get metadata for a specific data item
    async fn get_data_item(&self, data_id: &str) -> Option<DataItem>;
    
    /// Health check for the resource
    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Operations that can be performed on a resource
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    ListData { 
        categories: Vec<String>, 
        groups: Vec<String> 
    },
    ReadData { 
        data_id: String 
    },
    WriteData { 
        data_id: String, 
        value: serde_json::Value 
    },
    HasDataItem { 
        data_id: String 
    },
    GetDataItem { 
        data_id: String 
    },
    HealthCheck,
}

/// Results from resource operations
#[derive(Debug)]
pub enum OperationResult {
    DataItems(Vec<DataItem>),
    DataValue(serde_json::Value),
    Success,
    Bool(bool),
    DataItem(Option<DataItem>),
    Health(HealthStatus),
}