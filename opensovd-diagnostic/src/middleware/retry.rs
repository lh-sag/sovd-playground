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
use std::time::Duration;
use tokio::time::sleep;

use crate::resources::{DataError, OperationResult, ResourceOperation};
use super::{Next, ResourceMiddleware};

/// Middleware that retries failed operations with exponential backoff
pub struct RetryMiddleware {
    max_attempts: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl RetryMiddleware {
    /// Create a new retry middleware
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
        }
    }
    
    /// Create with custom backoff settings
    pub fn with_backoff(mut self, initial_ms: u64, max_ms: u64) -> Self {
        self.initial_backoff_ms = initial_ms;
        self.max_backoff_ms = max_ms;
        self
    }
    
    /// Check if error is retryable
    fn is_retryable(error: &DataError) -> bool {
        match error {
            DataError::InternalError(_) => true,
            DataError::DataNotFound(_) => false,
            DataError::ReadOnly(_) => false,
            DataError::AccessDenied(_) => false,
            DataError::InvalidData(_) => false,
        }
    }
}

#[async_trait]
impl ResourceMiddleware for RetryMiddleware {
    async fn process(
        &self,
        operation: ResourceOperation,
        next: Next<'_>,
    ) -> Result<OperationResult, DataError> {
        let mut attempt = 0;
        let mut backoff = self.initial_backoff_ms;
        
        loop {
            match next.clone().run(operation.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.max_attempts && Self::is_retryable(&e) => {
                    attempt += 1;
                    sleep(Duration::from_millis(backoff)).await;
                    
                    // Exponential backoff
                    backoff = (backoff * 2).min(self.max_backoff_ms);
                }
                Err(e) => return Err(e),
            }
        }
    }
}