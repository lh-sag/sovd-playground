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

use opensovd_diagnostic::resources::data::DataError;
use opensovd_models::entity::{DataErrorResponse, DataResourceItem};
use opensovd_models::error::{ErrorCode, GenericError};

use crate::response::ApiError;

/// Convert diagnostic DataError to API DataErrorResponse
pub fn data_error_to_response(data_error: DataError) -> DataErrorResponse {
    // Convert our simplified DataError to appropriate ErrorCode and vendor error
    let (error_code, vendor_code, message) = match data_error {
        DataError::DataNotFound(id) => (
            ErrorCode::VendorSpecific,
            Some(serde_json::json!("data-not-found")),
            format!("Data item not found: {id}"),
        ),
        DataError::ReadOnly(id) => (
            ErrorCode::InsufficientAccessRights,
            None,
            format!("Data item is read-only: {id}"),
        ),
        DataError::AccessDenied(id) => (
            ErrorCode::InsufficientAccessRights,
            None,
            format!("Access denied: {id}"),
        ),
        DataError::InvalidData(msg) => (ErrorCode::IncompleteRequest, None, format!("Invalid data: {msg}")),
        DataError::InternalError(msg) => (ErrorCode::SovdServerFailure, None, format!("Internal error: {msg}")),
    };

    DataErrorResponse {
        path: None, // Our simplified error doesn't track paths
        error: GenericError {
            error_code,
            message,
            vendor_code,
            transaction_id: None,
            parameters: None, // No additional parameters
        },
    }
}

/// Convert diagnostic DataError to ApiError
impl From<DataError> for ApiError {
    fn from(data_error: DataError) -> Self {
        match data_error {
            DataError::DataNotFound(msg) => ApiError::not_found(msg),
            DataError::ReadOnly(msg) => ApiError::bad_request(format!("Cannot write to read-only data: {}", msg)),
            DataError::AccessDenied(msg) => ApiError::forbidden(msg),
            DataError::InvalidData(msg) => ApiError::bad_request(msg),
            DataError::InternalError(msg) => ApiError::server_failure(msg),
        }
    }
}

/// Convert DataItem to API DataResourceItem
pub fn data_item_to_resource_item(item: opensovd_diagnostic::resources::data::DataItem) -> DataResourceItem {
    DataResourceItem {
        id: item.id,
        name: item.name,
        translation_id: None, // Our DataItem doesn't have translation_id
        category: item.category,
        groups: item.groups,
        tags: item.tags,
    }
}

/// Convert multiple DataItems to API DataResourceItems
pub fn data_items_to_resource_items(
    items: Vec<opensovd_diagnostic::resources::data::DataItem>,
) -> Vec<DataResourceItem> {
    items.into_iter().map(data_item_to_resource_item).collect()
}

#[cfg(test)]
mod tests {
    use opensovd_models::error::ErrorCode;

    use super::*;

    #[test]
    fn test_data_error_to_api_error() {
        // Test DataNotFound -> NotFound
        let error: ApiError = DataError::DataNotFound("Item not found".to_string()).into();
        assert_eq!(error.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(error.0.vendor_code, Some(crate::response::VendorError::NotFound));
        assert_eq!(error.0.message, "Item not found");

        // Test ReadOnly -> BadRequest
        let error: ApiError = DataError::ReadOnly("VIN".to_string()).into();
        assert_eq!(error.0.error_code, ErrorCode::IncompleteRequest);
        assert_eq!(error.0.message, "Cannot write to read-only data: VIN");

        // Test AccessDenied -> Forbidden
        let error: ApiError = DataError::AccessDenied("No permission".to_string()).into();
        assert_eq!(error.0.error_code, ErrorCode::InsufficientAccessRights);
        assert_eq!(error.0.message, "No permission");

        // Test InvalidData -> BadRequest
        let error: ApiError = DataError::InvalidData("Invalid format".to_string()).into();
        assert_eq!(error.0.error_code, ErrorCode::IncompleteRequest);
        assert_eq!(error.0.message, "Invalid format");

        // Test InternalError -> ServerFailure
        let error: ApiError = DataError::InternalError("Database error".to_string()).into();
        assert_eq!(error.0.error_code, ErrorCode::SovdServerFailure);
        assert_eq!(error.0.message, "Database error");
    }

    #[test]
    fn test_data_error_to_response() {
        let data_error = DataError::ReadOnly("read_only_item".to_string());

        let response = data_error_to_response(data_error);
        assert_eq!(response.path, None); // Our simplified error doesn't track paths
        assert_eq!(response.error.message, "Data item is read-only: read_only_item");
        assert_eq!(response.error.error_code, ErrorCode::InsufficientAccessRights);
    }

    #[test]
    fn test_data_item_conversion() {
        let diagnostic_item = opensovd_diagnostic::resources::data::DataItem {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            category: "currentData".to_string(), // Our DataItem uses String category
            groups: vec!["engine".to_string()],
            tags: vec!["temperature".to_string()],
        };

        let api_item = data_item_to_resource_item(diagnostic_item);

        assert_eq!(api_item.id, "test-id");
        assert_eq!(api_item.name, "Test Item");
        assert_eq!(api_item.translation_id, None); // Our DataItem doesn't have translation_id
        assert_eq!(api_item.category, "currentData");
        assert_eq!(api_item.groups, vec!["engine"]);
        assert_eq!(api_item.tags, vec!["temperature"]);
    }
}
