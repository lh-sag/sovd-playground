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
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionResponse<T = VendorInfo> {
    pub sovd_info: Vec<Info<T>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info<T = VendorInfo> {
    pub version: String,
    pub base_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_info: Option<T>,
}

// JsonSchema implementations for generic structs
#[cfg(feature = "jsonschema")]
impl<T> crate::JsonSchema for VersionResponse<T>
where
    T: crate::JsonSchema,
{
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error> {
        use serde_json::json;
        Ok(json!({
            "type": "object",
            "properties": {
                "sovd_info": {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/Info"
                    }
                }
            },
            "required": ["sovd_info"],
            "definitions": {
                "Info": Info::<T>::schema()?
            }
        }))
    }
}

#[cfg(feature = "jsonschema")]
impl<T> crate::JsonSchema for Info<T>
where
    T: crate::JsonSchema,
{
    fn schema() -> std::result::Result<serde_json::Value, serde_json::Error> {
        use serde_json::json;
        Ok(json!({
            "type": "object",
            "properties": {
                "version": { "type": "string" },
                "base_uri": { "type": "string" },
                "vendor_info": T::schema()?
            },
            "required": ["version", "base_uri"]
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct VendorInfo {
    pub version: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "jsonschema")]
    mod jsonschema_tests {
        use super::*;
        use crate::JsonSchema;

        #[test]
        fn test_info_jsonschema() {
            // Test schema for Info with a simple type that implements JsonSchema
            #[derive(Debug)]
            struct TestVendor;

            impl crate::JsonSchema for TestVendor {
                fn schema() -> Result<serde_json::Value, serde_json::Error> {
                    use serde_json::json;
                    Ok(json!({
                        "type": "object",
                        "properties": {
                            "test": { "type": "string" }
                        }
                    }))
                }
            }

            let schema = <Info<TestVendor> as JsonSchema>::schema().unwrap();

            // Verify the schema structure
            assert_eq!(schema["type"], "object");
            assert!(schema["properties"].is_object());
            assert_eq!(schema["properties"]["version"]["type"], "string");
            assert_eq!(schema["properties"]["base_uri"]["type"], "string");
            assert!(schema["properties"]["vendor_info"].is_object());

            // Verify required fields
            let required = schema["required"].as_array().unwrap();
            assert!(required.contains(&serde_json::json!("version")));
            assert!(required.contains(&serde_json::json!("base_uri")));
            assert_eq!(required.len(), 2); // vendor_info should not be required
        }

        #[test]
        fn test_version_response_jsonschema() {
            // Test schema for VersionResponse
            #[derive(Debug)]
            struct TestVendor;

            impl crate::JsonSchema for TestVendor {
                fn schema() -> Result<serde_json::Value, serde_json::Error> {
                    use serde_json::json;
                    Ok(json!({
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }))
                }
            }

            let schema = <VersionResponse<TestVendor> as JsonSchema>::schema().unwrap();

            // Verify the schema structure
            assert_eq!(schema["type"], "object");
            assert!(schema["properties"].is_object());
            assert_eq!(schema["properties"]["sovd_info"]["type"], "array");
            assert!(schema["properties"]["sovd_info"]["items"].is_object());
            assert_eq!(schema["properties"]["sovd_info"]["items"]["$ref"], "#/definitions/Info");

            // Verify required fields
            let required = schema["required"].as_array().unwrap();
            assert!(required.contains(&serde_json::json!("sovd_info")));
            assert_eq!(required.len(), 1);

            // Verify definitions
            assert!(schema["definitions"].is_object());
            assert!(schema["definitions"]["Info"].is_object());
        }

        #[test]
        #[cfg(feature = "jsonschema-schemars")]
        fn test_jsonschema_with_vendor_info() {
            // VendorInfo already has JsonSchema implementation via schemars when feature is enabled
            let info_schema = <Info<VendorInfo> as JsonSchema>::schema().unwrap();

            // Verify the schema structure
            assert_eq!(info_schema["type"], "object");
            assert!(info_schema["properties"]["version"].is_object());
            assert!(info_schema["properties"]["base_uri"].is_object());
            assert!(info_schema["properties"]["vendor_info"].is_object());

            let response_schema = <VersionResponse<VendorInfo> as JsonSchema>::schema().unwrap();

            // Verify the complete schema structure
            assert_eq!(response_schema["type"], "object");
            assert!(response_schema["definitions"]["Info"].is_object());
        }
    }
}
