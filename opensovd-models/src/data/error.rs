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

use serde::{Deserialize, Serialize};
use derive_more::{Display, Error};

/// Error type for data operations in the diagnostic layer
#[derive(Debug, Clone, Serialize, Deserialize, Display, Error)]
#[display("{}: {}", code, message)]
pub struct DataError {
    /// JSON pointer path to the error location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Error code identifying the type of error
    pub code: DataErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional parameters for the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Error codes for data operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum DataErrorCode {
    /// Data item not found
    #[serde(rename = "data-not-found")]
    #[display("data-not-found")]
    DataNotFound,
    /// Data item is read-only
    #[serde(rename = "data-read-only")]
    #[display("data-read-only")]
    DataReadOnly,
    /// Invalid data format
    #[serde(rename = "invalid-data-format")]
    #[display("invalid-data-format")]
    InvalidDataFormat,
    /// Access denied
    #[serde(rename = "access-denied")]
    #[display("access-denied")]
    AccessDenied,
    /// Internal error occurred
    #[serde(rename = "internal-error")]
    #[display("internal-error")]
    InternalError,
}

impl DataError {
    /// Creates a new DataError for a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            path: None,
            code: DataErrorCode::DataNotFound,
            message: message.into(),
            parameters: None,
        }
    }

    /// Creates a new DataError for a read-only error
    pub fn read_only(message: impl Into<String>) -> Self {
        Self {
            path: None,
            code: DataErrorCode::DataReadOnly,
            message: message.into(),
            parameters: None,
        }
    }

    /// Creates a new DataError for invalid data format
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self {
            path: None,
            code: DataErrorCode::InvalidDataFormat,
            message: message.into(),
            parameters: None,
        }
    }

    /// Creates a new DataError for access denied
    pub fn access_denied(message: impl Into<String>) -> Self {
        Self {
            path: None,
            code: DataErrorCode::AccessDenied,
            message: message.into(),
            parameters: None,
        }
    }

    /// Creates a new DataError for internal errors
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            path: None,
            code: DataErrorCode::InternalError,
            message: message.into(),
            parameters: None,
        }
    }

    /// Sets the JSON pointer path for this error
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets additional parameters for this error
    pub fn with_parameters(mut self, parameters: std::collections::HashMap<String, serde_json::Value>) -> Self {
        self.parameters = Some(parameters);
        self
    }
}