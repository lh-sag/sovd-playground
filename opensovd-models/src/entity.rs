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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct EntityId {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_id: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct EntityReference {
    #[serde(flatten)]
    pub entity: EntityId,
    pub href: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityResponse {
    pub items: Vec<EntityReference>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct Resources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configurations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bulk-data")]
    pub bulk_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "data-list")]
    pub data_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faults: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "communication-logs")]
    pub communication_logs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "cyclic-subscriptions")]
    pub cyclic_subscriptions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct ComponentCapabilitiesResponse {
    #[serde(flatten)]
    pub entity: EntityId,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub variant: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcomponents: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "belongs-to")]
    pub belongs_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "depends-on")]
    pub depends_on: Option<String>,

    #[serde(flatten)]
    pub resources: Resources,
}
