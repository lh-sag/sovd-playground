// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

//! Conversion utilities between diagnostic layer types and server API types
//!
//! This module provides the bridging layer between sovd-diagnostic and
//! sovd-models without creating direct dependencies between them.

use sovd_diagnostic::{DataCategory, DataError, DataValue, Entity, EntityType};
use sovd_models::data::ReadValue;
use sovd_models::entity::{EntityId, EntityReference};

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

impl From<DataError> for ApiError {
    fn from(data_error: DataError) -> Self {
        let code = &data_error.error.error_code;
        let msg = &data_error.error.message;
        match code {
            sovd_models::error::ErrorCode::InsufficientAccessRights => ApiError::forbidden(msg.clone()),
            sovd_models::error::ErrorCode::IncompleteRequest => ApiError::bad_request(msg.clone()),
            sovd_models::error::ErrorCode::SovdServerFailure => ApiError::server_failure(msg.clone()),
            _ => ApiError::not_found(msg.clone()),
        }
    }
}

pub fn data_value_to_rest(value: DataValue, include_schema: bool) -> ReadValue {
    ReadValue {
        id: value.id.clone(),
        data: value.value.clone(),
        errors: value.errors,
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
        entity: EntityId {
            id: entity.id().to_string(),
            name: entity.name().to_string(),
            translation_id: entity.translation_id().map(|s| s.to_string()),
        },
        href,
        tags: entity.tags().to_vec(),
    }
}
