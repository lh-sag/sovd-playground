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

use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::*;

    #[test]
    fn test_not_found_error_creation() {
        let error = DataError::not_found("Item not found");

        assert_eq!(error.code, DataErrorCode::DataNotFound);
        assert_eq!(error.message, "Item not found");
        assert!(error.path.is_none());
        assert!(error.parameters.is_none());
    }

    #[test]
    fn test_read_only_error_creation() {
        let error = DataError::read_only("Cannot modify read-only data");

        assert_eq!(error.code, DataErrorCode::DataReadOnly);
        assert_eq!(error.message, "Cannot modify read-only data");
        assert!(error.path.is_none());
        assert!(error.parameters.is_none());
    }

    #[test]
    fn test_invalid_format_error_creation() {
        let error = DataError::invalid_format("Invalid JSON format");

        assert_eq!(error.code, DataErrorCode::InvalidDataFormat);
        assert_eq!(error.message, "Invalid JSON format");
        assert!(error.path.is_none());
        assert!(error.parameters.is_none());
    }

    #[test]
    fn test_access_denied_error_creation() {
        let error = DataError::access_denied("Insufficient permissions");

        assert_eq!(error.code, DataErrorCode::AccessDenied);
        assert_eq!(error.message, "Insufficient permissions");
        assert!(error.path.is_none());
        assert!(error.parameters.is_none());
    }

    #[test]
    fn test_internal_error_creation() {
        let error = DataError::internal_error("Internal server error");

        assert_eq!(error.code, DataErrorCode::InternalError);
        assert_eq!(error.message, "Internal server error");
        assert!(error.path.is_none());
        assert!(error.parameters.is_none());
    }

    #[test]
    fn test_with_path_builder() {
        let error = DataError::not_found("Item not found").with_path("/data/items/42");

        assert_eq!(error.path, Some("/data/items/42".to_string()));
        assert_eq!(error.code, DataErrorCode::DataNotFound);
        assert_eq!(error.message, "Item not found");
    }

    #[test]
    fn test_with_parameters_builder() {
        let mut params = HashMap::new();
        params.insert("item_id".to_string(), json!("42"));
        params.insert("expected_type".to_string(), json!("string"));

        let error = DataError::invalid_format("Type mismatch").with_parameters(params.clone());

        assert!(error.parameters.is_some());
        let error_params = error.parameters.unwrap();
        assert_eq!(error_params.get("item_id"), Some(&json!("42")));
        assert_eq!(error_params.get("expected_type"), Some(&json!("string")));
    }

    #[test]
    fn test_chaining_builders() {
        let mut params = HashMap::new();
        params.insert("retry_after".to_string(), json!(30));

        let error = DataError::access_denied("Rate limit exceeded")
            .with_path("/api/data")
            .with_parameters(params);

        assert_eq!(error.code, DataErrorCode::AccessDenied);
        assert_eq!(error.message, "Rate limit exceeded");
        assert_eq!(error.path, Some("/api/data".to_string()));
        assert!(error.parameters.is_some());
        assert_eq!(error.parameters.unwrap().get("retry_after"), Some(&json!(30)));
    }

    #[test]
    fn test_display_implementation() {
        let error = DataError::not_found("Component not found");
        let display_str = format!("{error}");

        assert_eq!(display_str, "data-not-found: Component not found");
    }

    #[test]
    fn test_display_for_all_error_codes() {
        let test_cases = vec![
            (DataErrorCode::DataNotFound, "data-not-found"),
            (DataErrorCode::DataReadOnly, "data-read-only"),
            (DataErrorCode::InvalidDataFormat, "invalid-data-format"),
            (DataErrorCode::AccessDenied, "access-denied"),
            (DataErrorCode::InternalError, "internal-error"),
        ];

        for (code, expected_display) in test_cases {
            assert_eq!(format!("{code}"), expected_display);
        }
    }

    #[test]
    fn test_serialization() {
        let mut params = HashMap::new();
        params.insert("field".to_string(), json!("temperature"));

        let error = DataError {
            path: Some("/sensors/temp1".to_string()),
            code: DataErrorCode::InvalidDataFormat,
            message: "Invalid temperature value".to_string(),
            parameters: Some(params),
        };

        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["path"], "/sensors/temp1");
        assert_eq!(json["code"], "invalid-data-format");
        assert_eq!(json["message"], "Invalid temperature value");
        assert_eq!(json["parameters"]["field"], "temperature");
    }

    #[test]
    fn test_deserialization() {
        let json = json!({
            "path": "/data/items/1",
            "code": "data-not-found",
            "message": "Item with ID 1 not found",
            "parameters": {
                "item_id": 1,
                "timestamp": "2024-01-01T00:00:00Z"
            }
        });

        let error: DataError = serde_json::from_value(json).unwrap();

        assert_eq!(error.path, Some("/data/items/1".to_string()));
        assert_eq!(error.code, DataErrorCode::DataNotFound);
        assert_eq!(error.message, "Item with ID 1 not found");
        assert!(error.parameters.is_some());

        let params = error.parameters.unwrap();
        assert_eq!(params.get("item_id"), Some(&json!(1)));
        assert_eq!(params.get("timestamp"), Some(&json!("2024-01-01T00:00:00Z")));
    }

    #[test]
    fn test_serialization_skips_none_fields() {
        let error = DataError::internal_error("Server error");
        let json = serde_json::to_value(&error).unwrap();

        // path and parameters should not be present when None
        assert!(!json.as_object().unwrap().contains_key("path"));
        assert!(!json.as_object().unwrap().contains_key("parameters"));
        assert!(json.as_object().unwrap().contains_key("code"));
        assert!(json.as_object().unwrap().contains_key("message"));
    }

    #[test]
    fn test_error_code_equality() {
        assert_eq!(DataErrorCode::DataNotFound, DataErrorCode::DataNotFound);
        assert_ne!(DataErrorCode::DataNotFound, DataErrorCode::DataReadOnly);
    }

    #[test]
    fn test_builder_pattern_with_string_types() {
        // Test that various string types work with the builder methods
        let string = String::from("Dynamic path");
        let str_ref = "Static path";

        let error1 = DataError::not_found("Not found").with_path(string);
        let error2 = DataError::not_found("Not found").with_path(str_ref);

        assert!(error1.path.is_some());
        assert!(error2.path.is_some());
    }

    #[test]
    fn test_complex_parameters() {
        let mut params = HashMap::new();
        params.insert(
            "nested".to_string(),
            json!({
                "level1": {
                    "level2": {
                        "value": 42
                    }
                }
            }),
        );
        params.insert("array".to_string(), json!([1, 2, 3]));
        params.insert("null_value".to_string(), json!(null));

        let error = DataError::invalid_format("Complex error").with_parameters(params.clone());

        let error_params = error.parameters.unwrap();
        assert!(error_params.get("nested").unwrap()["level1"]["level2"]["value"].is_number());
        assert!(error_params.get("array").unwrap().is_array());
        assert!(error_params.get("null_value").unwrap().is_null());
    }
}
