// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

//! Conversion utilities between diagnostic layer types and server API types
//!
//! This module provides the bridging layer between sovd-diagnostic and
//! sovd-models without creating direct dependencies between them.

use sovd_diagnostic::{DataCategory, DataError, DataValue, Entity, EntityType, ValueMetaData};
use sovd_models::data::{ReadValue, ValueMetaData as ModelValueMetaData};
use sovd_models::entity::{EntityId as ModelEntityId, EntityReference};
use sovd_models::error::{ErrorCode, GenericError};

use crate::response::ApiError;

pub fn parse_categories(cats: Option<&[String]>) -> Vec<DataCategory> {
    cats.map(|c| c.iter().filter_map(|s| parse_single_category(s)).collect())
        .unwrap_or_default()
}

pub fn parse_single_category(s: &str) -> Option<DataCategory> {
    match s {
        "identData" => Some(DataCategory::IdentData),
        "currentData" => Some(DataCategory::CurrentData),
        "storedData" => Some(DataCategory::StoredData),
        "sysInfo" => Some(DataCategory::SysInfo),
        // Vendor categories must start with "x-" but not "x-sovd-"
        s if s.starts_with("x-") && !s.starts_with("x-sovd-") => Some(DataCategory::Vendor(s.to_string())),
        _ => None,
    }
}

/// Convert diagnostic DataCategory to models DataCategory
pub fn diagnostic_category_to_models(cat: DataCategory) -> sovd_models::data::DataCategory {
    match cat {
        DataCategory::IdentData => sovd_models::data::DataCategory::IdentData,
        DataCategory::CurrentData => sovd_models::data::DataCategory::CurrentData,
        DataCategory::StoredData => sovd_models::data::DataCategory::StoredData,
        DataCategory::SysInfo => sovd_models::data::DataCategory::SysInfo,
        DataCategory::Vendor(s) => {
            // Use new_vendor to validate, but fallback to direct Vendor if validation would fail
            sovd_models::data::DataCategory::new_vendor(s.clone()).unwrap_or(sovd_models::data::DataCategory::Vendor(s))
        }
    }
}

pub(crate) fn data_error_to_response(data_error: DataError) -> sovd_models::data::DataError {
    let (error_code, vendor_code, message) = match data_error {
        DataError::NotFound(id) => (
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
        DataError::Internal(msg) => (ErrorCode::SovdServerFailure, None, format!("Internal error: {msg}")),
    };

    sovd_models::data::DataError {
        path: String::new(), // Our simplified error doesn't track paths
        error: GenericError {
            error_code,
            message,
            vendor_code,
            translation_id: None,
            parameters: None, // No additional parameters
        },
    }
}

/// Convert diagnostic DataError to ApiError
impl From<DataError> for ApiError {
    fn from(data_error: DataError) -> Self {
        match data_error {
            DataError::NotFound(msg) => ApiError::not_found(msg),
            DataError::ReadOnly(msg) => ApiError::bad_request(format!("Cannot write to read-only data: {msg}")),
            DataError::AccessDenied(msg) => ApiError::forbidden(msg),
            DataError::InvalidData(msg) => ApiError::bad_request(msg),
            DataError::Internal(msg) => ApiError::server_failure(msg),
        }
    }
}

pub fn data_value_to_rest(value: DataValue, include_schema: bool) -> ReadValue {
    ReadValue {
        id: value.id.clone(),
        data: value.value.clone(),
        errors: value.errors.into_iter().map(data_error_to_response).collect(),
        schema: if include_schema {
            Some(generate_schema_for_value(&value.id, &value.value))
        } else {
            None
        },
    }
}

fn generate_schema_for_value(_id: &str, value: &serde_json::Value) -> serde_json::Value {
    use serde_json::json;

    match value {
        serde_json::Value::Bool(_) => json!({
            "type": "boolean"
        }),
        serde_json::Value::Number(_) => json!({
            "type": "number"
        }),
        serde_json::Value::String(_) => json!({
            "type": "string"
        }),
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first() {
                json!({
                    "type": "array",
                    "items": generate_schema_for_value("", first)
                })
            } else {
                json!({
                    "type": "array"
                })
            }
        }
        serde_json::Value::Object(_) => json!({
            "type": "object"
        }),
        serde_json::Value::Null => json!({
            "type": "null"
        }),
    }
}

pub(crate) fn value_metadata_to_model(item: ValueMetaData) -> ModelValueMetaData {
    ModelValueMetaData {
        id: item.id,
        name: item.name,
        translation_id: None,
        category: diagnostic_category_to_models(item.category),
        groups: item.groups,
        tags: item.tags,
    }
}

pub(crate) fn value_metadata_to_models(items: Vec<ValueMetaData>) -> Vec<ModelValueMetaData> {
    items.into_iter().map(value_metadata_to_model).collect()
}

pub fn entity_to_reference(entity: &dyn Entity, base_uri: &str, entity_type: EntityType) -> EntityReference {
    let collection = match entity_type {
        EntityType::Component => "components",
        EntityType::SovdServer => "", // Root has no collection path
    };

    let href = if collection.is_empty() {
        // Root SovdServer gets base URI only
        base_uri.to_string()
    } else {
        format!("{}/v1/{}/{}", base_uri.trim_end_matches('/'), collection, entity.id())
    };

    EntityReference {
        entity: ModelEntityId {
            id: entity.id().to_string(),
            name: entity.name().to_string(),
            translation_id: entity.translation_id().map(|s| s.to_string()),
        },
        href,
        tags: entity.tags().to_vec(),
    }
}
