//
// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
//

pub mod data;
pub mod entity;
pub mod error;
pub mod version;

/// A trait for types that can generate JSON schemas.
#[cfg(feature = "jsonschema")]
pub trait JsonSchema {
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error>;
}

/// Implementation for types that implement `schemars::JsonSchema`
#[cfg(feature = "jsonschema-schemars")]
impl<T> JsonSchema for T
where
    T: schemars::JsonSchema,
{
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error> {
        let schema = schemars::schema_for!(T);
        Ok(schema.to_value())
    }
}

/// Implementation for serde types when schemars is not available
/// This provides a basic schema using serde_json introspection
#[cfg(all(feature = "jsonschema", not(feature = "jsonschema-schemars")))]
impl<T> JsonSchema for T
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Default,
{
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error> {
        // Create a basic JSON schema structure for serde types
        use serde_json::json;
        Ok(json!({
            "type": "object",
            "description": "Auto-generated schema for serde type"
        }))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IncludeSchemaParam {
    #[serde(rename = "include-schema", default)]
    pub include_schema: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TagsParam {
    pub tags: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}
