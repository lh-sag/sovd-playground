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
