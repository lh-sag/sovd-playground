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

use serde::{Deserialize, Serialize};

use crate::data::DataCategory;

/// Data item metadata with generic category support
///
/// # Type Parameters
/// * `V` - The vendor extension type (defaults to String)
///
/// # Examples
/// ```rust
/// use opensovd_models::data::{DataItem, DataCategory};
///
/// // String-based vendor categories
/// let item = DataItem {
///     id: "temperature".to_string(),
///     name: "Engine Temperature".to_string(),
///     translation_id: None,
///     category: DataCategory::Vendor("x-liebherr-engine".to_string()),
///     groups: vec!["engine".to_string()],
///     tags: vec!["temperature".to_string()],
/// };
///
/// // Standard categories
/// let standard_item = DataItem {
///     id: "pressure".to_string(),
///     name: "Hydraulic Pressure".to_string(),
///     translation_id: None,
///     category: DataCategory::CurrentData,
///     groups: vec!["hydraulics".to_string()],
///     tags: vec!["pressure".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataItem {
    pub id: String,
    pub name: String,
    pub translation_id: Option<String>,
    pub category: DataCategory,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
}
