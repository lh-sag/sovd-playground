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
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::entities::Component;
use crate::middleware::{cache::CacheMiddleware, retry::RetryMiddleware, logging::LoggingMiddleware};
use crate::resources::remote::RemoteResource;
use super::ComponentResolver;

/// Service discovery resolver that queries an external service registry
pub struct ServiceDiscoveryResolver {
    discovery_url: String,
    cache_ttl: u64,
    enable_retry: bool,
    client: Client,
}

impl ServiceDiscoveryResolver {
    /// Create a new service discovery resolver
    pub fn new(discovery_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            discovery_url,
            cache_ttl: 60,
            enable_retry: true,
            client,
        }
    }
    
    /// Set cache TTL in seconds
    pub fn with_cache_ttl(mut self, seconds: u64) -> Self {
        self.cache_ttl = seconds;
        self
    }
    
    /// Enable/disable retry
    pub fn with_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }
}

#[derive(Serialize, Deserialize)]
struct ComponentInfo {
    id: String,
    name: String,
    url: String,
    #[serde(default)]
    headers: HashMap<String, String>,
}

#[async_trait]
impl ComponentResolver for ServiceDiscoveryResolver {
    async fn resolve(&self, id: &str) -> Option<Component> {
        // Query service discovery for component
        let url = format!("{}/components/{}", self.discovery_url, id);
        
        let res = self.client
            .get(&url)
            .send()
            .await
            .ok()?;
            
        if !res.status().is_success() {
            return None;
        }
        
        let info: ComponentInfo = res.json().await.ok()?;
        
        // Build remote component
        let remote = RemoteResource::new(info.url)
            .with_headers(info.headers)
            .with_timeout(5000);
            
        let mut component = Component::builder(info.id, info.name)
            .with_resource(Box::new(remote));
            
        // Add middleware
        component = component.with_middleware(Arc::new(LoggingMiddleware::new()));
        
        if self.cache_ttl > 0 {
            component = component.with_middleware(Arc::new(CacheMiddleware::new(self.cache_ttl)));
        }
        
        if self.enable_retry {
            component = component.with_middleware(Arc::new(RetryMiddleware::new(3)));
        }
        
        Some(component.build())
    }
    
    fn priority(&self) -> i32 {
        -10 // Lower priority than static config
    }
}

/// DNS-based resolver using SRV records
pub struct DnsResolver {
    service_prefix: String,
    cache_ttl: u64,
}

impl DnsResolver {
    /// Create a new DNS resolver
    pub fn new(service_prefix: String) -> Self {
        Self {
            service_prefix,
            cache_ttl: 60,
        }
    }
}

#[async_trait]
impl ComponentResolver for DnsResolver {
    async fn resolve(&self, id: &str) -> Option<Component> {
        // DNS SRV lookup would go here
        // For now, this is a placeholder
        
        // Example: _sovd._tcp.engine-ecu.local
        let _srv_name = format!("{}.{}.local", self.service_prefix, id);
        
        // In a real implementation, we would:
        // 1. Query DNS for SRV records
        // 2. Get host and port
        // 3. Build RemoteResource with discovered endpoint
        
        // Placeholder return
        None
    }
    
    fn priority(&self) -> i32 {
        -20 // Lower priority than service discovery
    }
}