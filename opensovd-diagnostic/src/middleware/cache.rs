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
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::resources::{DataError, OperationResult, ResourceOperation};
use super::{Next, ResourceMiddleware};

/// Cache entry with timestamp
struct CacheEntry {
    timestamp: Instant,
    value: serde_json::Value,
}

/// Middleware that caches read operations
pub struct CacheMiddleware {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl CacheMiddleware {
    /// Create a new cache middleware with specified TTL
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    /// Clear expired entries from cache
    async fn cleanup(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, entry| now.duration_since(entry.timestamp) < self.ttl);
    }
}

#[async_trait]
impl ResourceMiddleware for CacheMiddleware {
    async fn process(
        &self,
        operation: ResourceOperation,
        next: Next<'_>,
    ) -> Result<OperationResult, DataError> {
        match &operation {
            ResourceOperation::ReadData { data_id } => {
                // Check cache first
                {
                    let cache = self.cache.read().await;
                    if let Some(entry) = cache.get(data_id) {
                        if Instant::now().duration_since(entry.timestamp) < self.ttl {
                            // Cache hit
                            return Ok(OperationResult::DataValue(entry.value.clone()));
                        }
                    }
                }
                
                // Cache miss or expired - call next - clone the operation
                let result = next.run(operation.clone()).await?;
                
                // Update cache with new value
                if let OperationResult::DataValue(ref value) = result {
                    let mut cache = self.cache.write().await;
                    cache.insert(data_id.clone(), CacheEntry {
                        timestamp: Instant::now(),
                        value: value.clone(),
                    });
                    
                    // Periodically cleanup old entries
                    if cache.len() > 100 {
                        drop(cache);
                        self.cleanup().await;
                    }
                }
                
                Ok(result)
            }
            ResourceOperation::WriteData { data_id, .. } => {
                // Invalidate cache on write
                self.cache.write().await.remove(data_id);
                next.run(operation.clone()).await
            }
            _ => {
                // Pass through for other operations
                next.run(operation.clone()).await
            }
        }
    }
}