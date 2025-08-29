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
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{DataError, DataItem, DataResource, HealthStatus, Resource};

/// Local resource implementation that wraps a DataResource
pub struct LocalResource {
    data: Arc<RwLock<Box<dyn DataResource>>>,
}

impl LocalResource {
    /// Create a new LocalResource from a DataResource
    pub fn new(data_resource: Box<dyn DataResource>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data_resource)),
        }
    }
}

#[async_trait]
impl Resource for LocalResource {
    async fn list_data_items(&self, categories: &[String], groups: &[String]) -> Result<Vec<DataItem>, DataError> {
        let data = self.data.read().await;
        Ok(data.list_data_items(categories, groups).await)
    }
    
    async fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError> {
        let data = self.data.read().await;
        data.read_data(data_id).await
    }
    
    async fn write_data(&self, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        let mut data = self.data.write().await;
        data.write_data(data_id, value).await
    }
    
    async fn has_data_item(&self, data_id: &str) -> bool {
        let data = self.data.read().await;
        data.has_data_item(data_id).await
    }
    
    async fn get_data_item(&self, data_id: &str) -> Option<DataItem> {
        let data = self.data.read().await;
        data.get_data_item(data_id).await
    }
    
    async fn health_check(&self) -> HealthStatus {
        // Local resources are always healthy if they exist
        HealthStatus::Healthy
    }
}