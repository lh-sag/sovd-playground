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
use std::time::Instant;

use crate::resources::{DataError, OperationResult, ResourceOperation};
use super::{Next, ResourceMiddleware};

/// Middleware that logs all resource operations
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    /// Create a new logging middleware
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ResourceMiddleware for LoggingMiddleware {
    async fn process(
        &self,
        operation: ResourceOperation,
        next: Next<'_>,
    ) -> Result<OperationResult, DataError> {
        let start = Instant::now();
        
        // Log the operation
        match &operation {
            ResourceOperation::ListData { categories, groups } => {
                tracing::debug!(
                    target: "resource",
                    categories = ?categories,
                    groups = ?groups,
                    "ListData operation started"
                );
            }
            ResourceOperation::ReadData { data_id } => {
                tracing::debug!(
                    target: "resource",
                    data_id = %data_id,
                    "ReadData operation started"
                );
            }
            ResourceOperation::WriteData { data_id, .. } => {
                tracing::debug!(
                    target: "resource",
                    data_id = %data_id,
                    "WriteData operation started"
                );
            }
            ResourceOperation::HasDataItem { data_id } => {
                tracing::debug!(
                    target: "resource",
                    data_id = %data_id,
                    "HasDataItem operation started"
                );
            }
            ResourceOperation::GetDataItem { data_id } => {
                tracing::debug!(
                    target: "resource",
                    data_id = %data_id,
                    "GetDataItem operation started"
                );
            }
            ResourceOperation::HealthCheck => {
                tracing::debug!(
                    target: "resource",
                    "HealthCheck operation started"
                );
            }
        }
        
        // Execute the operation
        let result = next.run(operation.clone()).await;
        
        // Log the result
        let elapsed = start.elapsed();
        match &result {
            Ok(_) => {
                tracing::debug!(
                    target: "resource",
                    elapsed_ms = elapsed.as_millis(),
                    "Operation completed successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    target: "resource",
                    elapsed_ms = elapsed.as_millis(),
                    error = %e,
                    "Operation failed"
                );
            }
        }
        
        result
    }
}