// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema-schemars", schemars(bound = "T: Serialize"))]
pub struct VersionResponse<T = VendorInfo> {
    pub sovd_info: Vec<Info<T>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema-schemars", schemars(bound = "T: Serialize"))]
pub struct Info<T = VendorInfo> {
    pub version: String,
    pub base_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "jsonschema-schemars", schemars(skip))]
    pub vendor_info: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct VendorInfo {
    pub version: String,
    pub name: String,
}
