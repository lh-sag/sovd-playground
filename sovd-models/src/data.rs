// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use derive_more::Display;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Category Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub enum DataCategory {
    /// Identification data - static information about the entity
    #[display("identData")]
    #[serde(rename = "identData")]
    IdentData,
    /// Current data - live/real-time values
    #[display("currentData")]
    #[serde(rename = "currentData")]
    CurrentData,
    /// Stored data - historical/logged values
    #[display("storedData")]
    #[serde(rename = "storedData")]
    StoredData,
    /// System information - diagnostic/status data
    #[display("sysInfo")]
    #[serde(rename = "sysInfo")]
    SysInfo,
    /// Vendor-specific category extension
    #[display("{}", _0)]
    Vendor(String),
}

impl DataCategory {
    pub fn new_vendor(category: impl Into<String>) -> Result<Self, DataCategoryError> {
        let category_str = category.into();
        if is_valid_vendor_category(&category_str) {
            Ok(DataCategory::Vendor(category_str))
        } else {
            Err(DataCategoryError::InvalidFormat(category_str))
        }
    }
}

/// Validates vendor category format: x-<extension>
///
/// Rules:
/// - Must start with "x-"
/// - Must not start with "x-sovd-"
/// - Extension can contain: a-z, 0-9, hyphens, underscores
/// - Must not end with hyphen
/// - Must not have consecutive hyphens
fn is_valid_vendor_category(category: &str) -> bool {
    if !category.starts_with("x-") {
        return false;
    }
    if category.starts_with("x-sovd-") {
        return false;
    }
    let extension = &category[2..];
    if extension.is_empty() {
        return false;
    }

    let mut prev_was_hyphen = false;
    for c in extension.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            return false;
        }
        // Check for consecutive hyphens
        if c == '-' {
            if prev_was_hyphen {
                return false;
            }
            prev_was_hyphen = true;
        } else {
            prev_was_hyphen = false;
        }
    }

    !extension.ends_with('-')
}

/// Error type for data category validation
#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum DataCategoryError {
    /// Invalid vendor category format
    #[display("Invalid vendor category format: '{}'.", _0)]
    InvalidFormat(#[error(ignore)] String),
}

/// Information about a data category as per ISO 17978-3 Section 7.9.2.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataCategoryInformation {
    /// The data category
    pub item: DataCategory,
    /// Optional translation identifier for the category name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_translation_id: Option<String>,
}

/// Response containing available data categories as per ISO 17978-3 Section 7.9.2.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataCategoryResponse {
    /// List of supported data categories
    pub items: Vec<DataCategoryInformation>,
}

// ============================================================================
// Data Group Types
// ============================================================================

/// Value group information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct ValueGroup {
    /// Unique identifier for the group across all categories
    pub id: String,
    /// The data category for which the group is defined
    pub category: DataCategory,
    /// Optional translation identifier for the category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_translation_id: Option<String>,
    /// Name of the group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Optional translation identifier for the group name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_translation_id: Option<String>,
}

/// Response containing available data groups
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataGroupResponse {
    /// List of value groups
    pub items: Vec<ValueGroup>,
}

/// Query parameters for data group listing
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataGroupQuery {
    /// Optional category filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Include OpenAPI schema in response
    #[serde(rename = "include-schema", default)]
    pub include_schema: bool,
}

// ============================================================================
// Value Metadata and Read Types
// ============================================================================

/// Value metadata as per ISO 17978-3 Section 7.9.3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct ValueMetaData {
    /// Unique identifier for the value
    pub id: String,
    /// Name of the data value
    pub name: String,
    /// Translation identifier for the name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_id: Option<String>,
    /// Category of the data value
    pub category: DataCategory,
    /// Group identifiers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    /// Tag identifiers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Read value response as per ISO 17978-3 Section 7.9.4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct ReadValue {
    /// Unique identifier for the value
    pub id: String,
    /// The value (type defined by schema)
    pub data: serde_json::Value,
    /// Errors if value represents an error condition
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<DataError>,
}

// ============================================================================
// Data Error Types
// ============================================================================

/// Error type for data operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataError {
    pub path: String,
    pub error: crate::error::GenericError,
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
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            error: crate::error::GenericError {
                error_code: crate::error::ErrorCode::VendorSpecific,
                vendor_code: Some(DataErrorCode::DataNotFound.to_string()),
                message: message.into(),
                translation_id: None,
                parameters: None,
            },
        }
    }

    pub fn read_only(message: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            error: crate::error::GenericError {
                error_code: crate::error::ErrorCode::InsufficientAccessRights,
                vendor_code: None,
                message: message.into(),
                translation_id: None,
                parameters: None,
            },
        }
    }

    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            error: crate::error::GenericError {
                error_code: crate::error::ErrorCode::InvalidResponseContent,
                vendor_code: None,
                message: message.into(),
                translation_id: None,
                parameters: None,
            },
        }
    }

    pub fn access_denied(message: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            error: crate::error::GenericError {
                error_code: crate::error::ErrorCode::InsufficientAccessRights,
                vendor_code: None,
                message: message.into(),
                translation_id: None,
                parameters: None,
            },
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            error: crate::error::GenericError {
                error_code: crate::error::ErrorCode::SovdServerFailure,
                vendor_code: None,
                message: message.into(),
                translation_id: None,
                parameters: None,
            },
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_parameters(mut self, parameters: std::collections::HashMap<String, serde_json::Value>) -> Self {
        self.error.parameters = Some(parameters);
        self
    }
}

// ============================================================================
// Query and Response Types for Data Resources
// ============================================================================

/// Query parameters for data resource listing as per ISO 17978-3 Section 7.9.3
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataResourceQuery {
    /// Filter by categories
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    /// Filter by groups
    #[serde(default)]
    pub groups: Option<Vec<String>>,
    /// Include OpenAPI schema in response
    #[serde(rename = "include-schema", default)]
    pub include_schema: bool,
}

/// Response for listing data resources as per ISO 17978-3 Section 7.9.3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataResourceResponse {
    /// List of data resource metadata
    pub items: Vec<ValueMetaData>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::*;

    // Category tests
    #[test]
    fn test_valid_vendor_categories() {
        // Test basic valid vendor categories
        assert!(DataCategory::new_vendor("x-vendor").is_ok());
        assert!(DataCategory::new_vendor("x-vendor-engine").is_ok());
        assert!(DataCategory::new_vendor("x-test123").is_ok());
        assert!(DataCategory::new_vendor("x-under_score").is_ok());

        // Test that valid vendor categories display correctly
        let vendor = DataCategory::new_vendor("x-test").unwrap();
        assert_eq!(format!("{vendor}"), "x-test");
    }

    #[test]
    fn test_invalid_vendor_categories() {
        // Test missing prefix
        assert!(DataCategory::new_vendor("no-prefix").is_err());
        assert!(DataCategory::new_vendor("liebherr").is_err());

        // Test empty extension
        assert!(DataCategory::new_vendor("x-").is_err());

        // Test reserved x-sovd- prefix (ISO 17978-3)
        assert!(DataCategory::new_vendor("x-sovd-").is_err());
        assert!(DataCategory::new_vendor("x-sovd-custom").is_err());
        assert!(DataCategory::new_vendor("x-sovd-anything").is_err());

        // Test consecutive hyphens
        assert!(DataCategory::new_vendor("x-test--double").is_err());
        assert!(DataCategory::new_vendor("x---triple").is_err());

        // Test ending with hyphen
        assert!(DataCategory::new_vendor("x-test-").is_err());

        // Test uppercase characters
        assert!(DataCategory::new_vendor("x-UPPER").is_err());
        assert!(DataCategory::new_vendor("x-Mixed").is_err());

        // Test invalid characters
        assert!(DataCategory::new_vendor("x-space here").is_err());
        assert!(DataCategory::new_vendor("x-special@").is_err());
        assert!(DataCategory::new_vendor("x-dot.here").is_err());
        assert!(DataCategory::new_vendor("x-slash/here").is_err());

        // Test wrong prefix
        assert!(DataCategory::new_vendor("y-wrong").is_err());
        assert!(DataCategory::new_vendor("X-wrong").is_err());

        // Test completely invalid formats
        assert!(DataCategory::new_vendor("").is_err());
        assert!(DataCategory::new_vendor("x").is_err());
        assert!(DataCategory::new_vendor("-test").is_err());
    }

    // Error tests
    #[test]
    fn test_not_found_error_creation() {
        let error = DataError::not_found("Item not found");

        assert_eq!(error.error.error_code, crate::error::ErrorCode::VendorSpecific);
        assert_eq!(error.error.vendor_code, Some(DataErrorCode::DataNotFound.to_string()));
        assert_eq!(error.error.message, "Item not found");
        assert_eq!(error.path, "");
        assert!(error.error.parameters.is_none());
    }

    #[test]
    fn test_read_only_error_creation() {
        let error = DataError::read_only("Cannot modify read-only data");

        assert_eq!(
            error.error.error_code,
            crate::error::ErrorCode::InsufficientAccessRights
        );
        assert_eq!(error.error.message, "Cannot modify read-only data");
        assert_eq!(error.path, "");
        assert!(error.error.parameters.is_none());
    }

    #[test]
    fn test_invalid_format_error_creation() {
        let error = DataError::invalid_format("Invalid JSON format");

        assert_eq!(error.error.error_code, crate::error::ErrorCode::InvalidResponseContent);
        assert_eq!(error.error.message, "Invalid JSON format");
        assert_eq!(error.path, "");
        assert!(error.error.parameters.is_none());
    }

    #[test]
    fn test_access_denied_error_creation() {
        let error = DataError::access_denied("Insufficient permissions");

        assert_eq!(
            error.error.error_code,
            crate::error::ErrorCode::InsufficientAccessRights
        );
        assert_eq!(error.error.message, "Insufficient permissions");
        assert_eq!(error.path, "");
        assert!(error.error.parameters.is_none());
    }

    #[test]
    fn test_internal_error_creation() {
        let error = DataError::internal_error("Internal server error");

        assert_eq!(error.error.error_code, crate::error::ErrorCode::SovdServerFailure);
        assert_eq!(error.error.message, "Internal server error");
        assert_eq!(error.path, "");
        assert!(error.error.parameters.is_none());
    }

    #[test]
    fn test_with_path_builder() {
        let error = DataError::not_found("Item not found").with_path("/data/items/42");

        assert_eq!(error.path, "/data/items/42");
        assert_eq!(error.error.vendor_code, Some(DataErrorCode::DataNotFound.to_string()));
        assert_eq!(error.error.message, "Item not found");
    }

    #[test]
    fn test_with_parameters_builder() {
        let mut params = HashMap::new();
        params.insert("item_id".to_string(), json!("42"));
        params.insert("expected_type".to_string(), json!("string"));

        let error = DataError::invalid_format("Type mismatch").with_parameters(params.clone());

        assert!(error.error.parameters.is_some());
        let error_params = error.error.parameters.as_ref().unwrap();
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

        assert_eq!(
            error.error.error_code,
            crate::error::ErrorCode::InsufficientAccessRights
        );
        assert_eq!(error.error.message, "Rate limit exceeded");
        assert_eq!(error.path, "/api/data");
        assert!(error.error.parameters.is_some());
        assert_eq!(
            error.error.parameters.as_ref().unwrap().get("retry_after"),
            Some(&json!(30))
        );
    }

    #[test]
    fn test_serialization() {
        let mut params = HashMap::new();
        params.insert("field".to_string(), json!("temperature"));

        let error = DataError::invalid_format("Invalid temperature value")
            .with_path("/sensors/temp1")
            .with_parameters(params);

        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["path"], "/sensors/temp1");
        assert_eq!(json["error"]["error_code"], "invalid-response-content");
        assert_eq!(json["error"]["message"], "Invalid temperature value");
        assert_eq!(json["error"]["parameters"]["field"], "temperature");
    }

    #[test]
    fn test_serialization_skips_none_fields() {
        let error = DataError::internal_error("Server error");
        let json = serde_json::to_value(&error).unwrap();

        let error_obj = json["error"].as_object().unwrap();
        assert!(!error_obj.contains_key("parameters"));
        assert!(!error_obj.contains_key("translation_id"));
        assert!(error_obj.contains_key("error_code"));
        assert!(error_obj.contains_key("message"));
    }

    #[test]
    fn test_builder_pattern_with_string_types() {
        // Test that various string types work with the builder methods
        let string = String::from("Dynamic path");
        let str_ref = "Static path";

        let error1 = DataError::not_found("Not found").with_path(string);
        let error2 = DataError::not_found("Not found").with_path(str_ref);

        assert_eq!(error1.path, "Dynamic path");
        assert_eq!(error2.path, "Static path");
    }
}
