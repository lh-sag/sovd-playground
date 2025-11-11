// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

/// Generic capabilities response for any entity type
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct EntityCapabilitiesResponse {
    #[serde(flatten)]
    pub entity: EntityId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<HashMap<String, String>>,

    #[serde(flatten)]
    pub relationships: EntityRelationships,

    #[serde(flatten)]
    pub resources: Resources,
}

/// All possible entity relationships
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct EntityRelationships {
    // Entity collections (root level only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub areas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<String>,

    // Hierarchical relationships
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subareas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcomponents: Option<String>,

    // Cross-entity relationships
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "belongs-to")]
    pub belongs_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "depends-on")]
    pub depends_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "is-located-on")]
    pub is_located_on: Option<String>,
}

/// Request for writing data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataWriteRequest {
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Query parameters for entity listing
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct EntityQuery {
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(rename = "include-schema", default)]
    pub include_schema: bool,
}
