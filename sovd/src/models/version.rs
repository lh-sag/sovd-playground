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
pub struct VersionInfo<T = VendorInfo> {
    pub info: Vec<Info<T>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info<T = VendorInfo> {
    pub version: String,
    pub base_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_info: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorInfo {
    pub version: String,
    pub name: String,
}
