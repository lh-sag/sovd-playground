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
use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{DataError, DataItem, HealthStatus, Resource};

/// Configuration for a remote resource
#[derive(Debug, Clone)]
pub struct RemoteResource {
    /// Base URL of the remote SOVD endpoint
    pub url: String,
    /// Optional authentication headers
    pub headers: HashMap<String, String>,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
}

impl RemoteResource {
    /// Create a new RemoteResource
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
            timeout_ms: 5000,
        }
    }
    
    /// Set authentication headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
    
    /// Create a client with timeout
    fn create_client(&self) -> Result<Client, DataError> {
        Client::builder()
            .timeout(Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| DataError::InternalError(format!("Failed to create client: {}", e)))
    }
    
    /// Add headers to request builder
    fn add_headers(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }
        req.header("Content-Type", "application/json")
    }
}

#[derive(Serialize, Deserialize)]
struct DataResourceResponse {
    items: Vec<DataResourceItem>,
}

#[derive(Serialize, Deserialize)]
struct DataResourceItem {
    id: String,
    name: String,
    category: String,
    groups: Vec<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DataValueResponse {
    id: String,
    data: serde_json::Value,
    errors: Vec<ErrorResponse>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

#[async_trait]
impl Resource for RemoteResource {
    async fn list_data_items(&self, categories: &[String], groups: &[String]) -> Result<Vec<DataItem>, DataError> {
        let mut url = format!("{}/data", self.url);
        let mut query_parts = Vec::new();
        
        for cat in categories {
            query_parts.push(format!("categories[]={}", urlencoding::encode(cat)));
        }
        for group in groups {
            query_parts.push(format!("groups[]={}", urlencoding::encode(group)));
        }
        
        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }
        
        let client = self.create_client()?;
        let req = client.get(&url);
        let req = self.add_headers(req);
        
        let res = req.send().await
            .map_err(|e| DataError::InternalError(format!("Remote request failed: {}", e)))?;
            
        if !res.status().is_success() {
            return Err(DataError::InternalError(format!("Remote returned status: {}", res.status())));
        }
        
        let response: DataResourceResponse = res.json().await
            .map_err(|e| DataError::InvalidData(format!("Failed to parse response: {}", e)))?;
            
        Ok(response.items.into_iter().map(|item| DataItem {
            id: item.id,
            name: item.name,
            category: item.category,
            groups: item.groups,
            tags: item.tags,
        }).collect())
    }
    
    async fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError> {
        let url = format!("{}/data/{}", self.url, urlencoding::encode(data_id));
        
        let client = self.create_client()?;
        let req = client.get(&url);
        let req = self.add_headers(req);
        
        let res = req.send().await
            .map_err(|e| DataError::InternalError(format!("Remote request failed: {}", e)))?;
            
        if !res.status().is_success() {
            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(DataError::DataNotFound(data_id.to_string()));
            }
            return Err(DataError::InternalError(format!("Remote returned status: {}", res.status())));
        }
        
        let response: DataValueResponse = res.json().await
            .map_err(|e| DataError::InvalidData(format!("Failed to parse response: {}", e)))?;
            
        if !response.errors.is_empty() {
            let first_error = &response.errors[0];
            return Err(match first_error.code.as_str() {
                "DataNotFound" => DataError::DataNotFound(data_id.to_string()),
                "ReadOnly" => DataError::ReadOnly(data_id.to_string()),
                "AccessDenied" => DataError::AccessDenied(first_error.message.clone()),
                _ => DataError::InternalError(first_error.message.clone()),
            });
        }
        
        Ok(response.data)
    }
    
    async fn write_data(&self, data_id: &str, value: serde_json::Value) -> Result<(), DataError> {
        let url = format!("{}/data/{}", self.url, urlencoding::encode(data_id));
        
        #[derive(Serialize)]
        struct WriteRequest {
            data: serde_json::Value,
        }
        
        let client = self.create_client()?;
        let req = client.put(&url);
        let req = self.add_headers(req);
        
        let res = req.json(&WriteRequest { data: value }).send().await
            .map_err(|e| DataError::InternalError(format!("Remote request failed: {}", e)))?;
            
        if !res.status().is_success() {
            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(DataError::DataNotFound(data_id.to_string()));
            }
            if res.status() == reqwest::StatusCode::FORBIDDEN {
                return Err(DataError::ReadOnly(data_id.to_string()));
            }
            return Err(DataError::InternalError(format!("Remote returned status: {}", res.status())));
        }
        
        Ok(())
    }
    
    async fn has_data_item(&self, data_id: &str) -> bool {
        self.get_data_item(data_id).await.is_some()
    }
    
    async fn get_data_item(&self, data_id: &str) -> Option<DataItem> {
        // Try to get the specific item metadata
        let items = self.list_data_items(&[], &[]).await.ok()?;
        items.into_iter().find(|item| item.id == data_id)
    }
    
    async fn health_check(&self) -> HealthStatus {
        // Try a simple request to check if remote is reachable
        let url = format!("{}/data", self.url);
        
        let client = match self.create_client() {
            Ok(c) => c,
            Err(e) => return HealthStatus::Unhealthy(format!("Cannot create client: {}", e)),
        };
        
        let req = client.get(&url);
        let req = self.add_headers(req);
        
        match req.send().await {
            Ok(res) if res.status().is_success() => HealthStatus::Healthy,
            Ok(res) => HealthStatus::Degraded(format!("Remote returned status: {}", res.status())),
            Err(e) => HealthStatus::Unhealthy(format!("Cannot reach remote: {}", e)),
        }
    }
}