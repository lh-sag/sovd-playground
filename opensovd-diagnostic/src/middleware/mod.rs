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
use std::sync::Arc;

use crate::resources::{DataError, OperationResult, Resource, ResourceOperation};

pub mod cache;
pub mod retry;
pub mod logging;

/// Trait for middleware that can intercept resource operations
#[async_trait]
pub trait ResourceMiddleware: Send + Sync + 'static {
    /// Process an operation, potentially calling the next middleware in the chain
    async fn process(
        &self,
        operation: ResourceOperation,
        next: Next<'_>,
    ) -> Result<OperationResult, DataError>;
}

/// Represents the next middleware in the chain or the final resource
pub struct Next<'a> {
    middleware: &'a [Arc<dyn ResourceMiddleware>],
    resource: &'a dyn Resource,
}

impl<'a> Next<'a> {
    /// Create a new Next instance
    pub fn new(middleware: &'a [Arc<dyn ResourceMiddleware>], resource: &'a dyn Resource) -> Self {
        Self { middleware, resource }
    }
    
    /// Execute the operation, either passing to next middleware or to the resource
    pub async fn run(self, operation: ResourceOperation) -> Result<OperationResult, DataError> {
        if let Some((first, rest)) = self.middleware.split_first() {
            // Call the next middleware
            let next = Next::new(rest, self.resource);
            first.process(operation, next).await
        } else {
            // End of chain - execute on resource
            match operation {
                ResourceOperation::ListData { categories, groups } => {
                    let items = self.resource.list_data_items(&categories, &groups).await?;
                    Ok(OperationResult::DataItems(items))
                }
                ResourceOperation::ReadData { data_id } => {
                    let value = self.resource.read_data(&data_id).await?;
                    Ok(OperationResult::DataValue(value))
                }
                ResourceOperation::WriteData { data_id, value } => {
                    self.resource.write_data(&data_id, value).await?;
                    Ok(OperationResult::Success)
                }
                ResourceOperation::HasDataItem { data_id } => {
                    let exists = self.resource.has_data_item(&data_id).await;
                    Ok(OperationResult::Bool(exists))
                }
                ResourceOperation::GetDataItem { data_id } => {
                    let item = self.resource.get_data_item(&data_id).await;
                    Ok(OperationResult::DataItem(item))
                }
                ResourceOperation::HealthCheck => {
                    let health = self.resource.health_check().await;
                    Ok(OperationResult::Health(health))
                }
            }
        }
    }
}

impl<'a> Clone for Next<'a> {
    fn clone(&self) -> Self {
        Self {
            middleware: self.middleware,
            resource: self.resource,
        }
    }
}