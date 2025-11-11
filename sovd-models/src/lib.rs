// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

pub mod data;
pub mod entity;
pub mod error;
pub mod version;

/// A trait for types that can generate JSON schemas.
/// When jsonschema-schemars feature is enabled, also requires Serialize + Deserialize.
#[cfg(feature = "jsonschema")]
pub trait JsonSchema: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error>;
}

/// Implementation for types that implement `schemars::JsonSchema`
#[cfg(feature = "jsonschema-schemars")]
impl<T> JsonSchema for T
where
    T: schemars::JsonSchema + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error> {
        let schema = schemars::schema_for!(T);
        Ok(schema.to_value())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct IncludeSchemaQuery {
    #[serde(rename = "include-schema", default)]
    pub include_schema: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}
