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

//! Conversion utilities between diagnostic layer types and server API types
//!
//! This module provides the bridging layer between opensovd-diagnostic and
//! opensovd-models without creating direct dependencies between them.


use opensovd_models::data::{DataError, DataErrorCode, StringDataCategory, StringDataItem};
use opensovd_models::entity::{DataResourceItem, DataErrorResponse};
use opensovd_models::error::{ErrorCode, GenericError};

use crate::error::ApiError;

/// Convert diagnostic DataError to API DataErrorResponse
pub fn data_error_to_response(data_error: DataError) -> DataErrorResponse {
    // Convert DataErrorCode to appropriate ErrorCode and vendor error
    let (error_code, vendor_code) = match data_error.code {
        DataErrorCode::DataNotFound => (
            ErrorCode::VendorSpecific,
            Some(serde_json::json!("data-not-found")),
        ),
        DataErrorCode::DataReadOnly => (
            ErrorCode::InsufficientAccessRights,
            None,
        ),
        DataErrorCode::InvalidDataFormat => (
            ErrorCode::IncompleteRequest,
            None,
        ),
        DataErrorCode::AccessDenied => (
            ErrorCode::InsufficientAccessRights,
            None,
        ),
        DataErrorCode::InternalError => (
            ErrorCode::SovdServerFailure,
            None,
        ),
    };

    DataErrorResponse {
        path: data_error.path,
        error: GenericError {
            error_code,
            message: data_error.message,
            vendor_code,
            transaction_id: None,
            parameters: data_error.parameters,
        },
    }
}

/// Convert diagnostic DataError to ApiError
pub fn data_error_to_api_error(data_error: DataError) -> ApiError {
    match data_error.code {
        DataErrorCode::DataNotFound => {
            ApiError::not_found(data_error.message)
        }
        DataErrorCode::DataReadOnly => {
            ApiError::forbidden(data_error.message)
        }
        DataErrorCode::InvalidDataFormat => {
            ApiError::bad_request(data_error.message)
        }
        DataErrorCode::AccessDenied => {
            ApiError::forbidden(data_error.message)
        }
        DataErrorCode::InternalError => {
            ApiError::internal_error(data_error.message)
        }
    }
}

/// Convert StringDataCategory to string format for API responses
pub fn data_category_to_string(category: StringDataCategory) -> String {
    match category {
        StringDataCategory::IdentData => "identData".to_string(),
        StringDataCategory::CurrentData => "currentData".to_string(),
        StringDataCategory::StoredData => "storedData".to_string(),
        StringDataCategory::SysInfo => "sysInfo".to_string(),
        StringDataCategory::Vendor(vendor_cat) => vendor_cat,
    }
}

/// Parse string category to StringDataCategory enum
pub fn parse_data_category(category: &str) -> Option<StringDataCategory> {
    match category {
        "identData" => Some(StringDataCategory::IdentData),
        "currentData" => Some(StringDataCategory::CurrentData),
        "storedData" => Some(StringDataCategory::StoredData),
        "sysInfo" => Some(StringDataCategory::SysInfo),
        vendor_cat if vendor_cat.starts_with("x-") => {
            // Validate and create vendor category
            StringDataCategory::new_string_vendor(vendor_cat).ok()
        },
        _ => None,
    }
}

/// Convert StringDataItem to API DataResourceItem
pub fn data_item_to_resource_item(item: StringDataItem) -> DataResourceItem {
    DataResourceItem {
        id: item.id,
        name: item.name,
        translation_id: item.translation_id,
        category: data_category_to_string(item.category),
        groups: item.groups,
        tags: item.tags,
    }
}

/// Convert multiple StringDataItems to API DataResourceItems
pub fn data_items_to_resource_items(items: Vec<StringDataItem>) -> Vec<DataResourceItem> {
    items.into_iter().map(data_item_to_resource_item).collect()
}

/// Parse multiple category filters from API query to StringDataCategory enum
pub fn parse_category_filters(categories: &[String]) -> Vec<StringDataCategory> {
    categories
        .iter()
        .filter_map(|cat| parse_data_category(cat))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensovd_models::error::ErrorCode;

    #[test]
    fn test_data_error_conversion() {
        let data_error = DataError::not_found("Test item not found");
        let api_error = data_error_to_api_error(data_error.clone());
        
        // The conversion should result in a not found API error
        assert!(matches!(api_error.0.error_code, ErrorCode::VendorSpecific));
        assert_eq!(api_error.0.message, "Test item not found");
    }

    #[test]
    fn test_data_error_to_response() {
        let data_error = DataError::read_only("Read only item")
            .with_path("/data/test");
        
        let response = data_error_to_response(data_error);
        assert_eq!(response.path, Some("/data/test".to_string()));
        assert_eq!(response.error.message, "Read only item");
        assert_eq!(response.error.error_code, ErrorCode::InsufficientAccessRights);
    }

    #[test]
    fn test_category_conversions() {
        // Test category to string
        assert_eq!(data_category_to_string(StringDataCategory::IdentData), "identData");
        assert_eq!(data_category_to_string(StringDataCategory::CurrentData), "currentData");
        assert_eq!(data_category_to_string(StringDataCategory::StoredData), "storedData");
        assert_eq!(data_category_to_string(StringDataCategory::SysInfo), "sysInfo");
        assert_eq!(data_category_to_string(StringDataCategory::Vendor("x-liebherr-hydraulics".to_string())), "x-liebherr-hydraulics");

        // Test string to category
        assert_eq!(parse_data_category("identData"), Some(StringDataCategory::IdentData));
        assert_eq!(parse_data_category("currentData"), Some(StringDataCategory::CurrentData));
        assert_eq!(parse_data_category("storedData"), Some(StringDataCategory::StoredData));
        assert_eq!(parse_data_category("sysInfo"), Some(StringDataCategory::SysInfo));
        assert_eq!(parse_data_category("x-liebherr-engine"), Some(StringDataCategory::Vendor("x-liebherr-engine".to_string())));
        assert_eq!(parse_data_category("invalid"), None);
        assert_eq!(parse_data_category("x-invalid--format"), None); // Invalid vendor format
    }

    #[test]
    fn test_data_item_conversion() {
        let diagnostic_item = StringDataItem {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            translation_id: Some("test.item".to_string()),
            category: StringDataCategory::CurrentData,
            groups: vec!["engine".to_string()],
            tags: vec!["temperature".to_string()],
        };

        let api_item = data_item_to_resource_item(diagnostic_item);
        
        assert_eq!(api_item.id, "test-id");
        assert_eq!(api_item.name, "Test Item");
        assert_eq!(api_item.translation_id, Some("test.item".to_string()));
        assert_eq!(api_item.category, "currentData");
        assert_eq!(api_item.groups, vec!["engine"]);
        assert_eq!(api_item.tags, vec!["temperature"]);
    }

    #[test]
    fn test_parse_category_filters() {
        let categories = vec![
            "identData".to_string(),
            "currentData".to_string(),
            "invalid".to_string(),
            "storedData".to_string(),
        ];

        let parsed = parse_category_filters(&categories);
        assert_eq!(parsed.len(), 3); // "invalid" should be filtered out
        assert!(parsed.contains(&StringDataCategory::IdentData));
        assert!(parsed.contains(&StringDataCategory::CurrentData));
        assert!(parsed.contains(&StringDataCategory::StoredData));
        assert!(!parsed.contains(&StringDataCategory::SysInfo)); // Not in input
    }
}