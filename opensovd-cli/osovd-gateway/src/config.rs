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
use std::path::Path;

use opensovd_diagnostic::{
    resources::data::DataItem,
    registry::ComponentRegistry,
    entities::Component,
    resources::{LocalResource, RemoteResource},
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "config-entities")]
use crate::hashmap_data_resource::{DataValue, HashMapDataResource};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub components: Vec<ComponentConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt: Option<JwtConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtConfig {
    pub public_key_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub data: Vec<DataItemConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoteConfig {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataItemConfig {
    pub id: String,
    pub name: String,
    pub category: String,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
    pub value: serde_json::Value,
    pub writable: bool,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_str(&contents)
    }

    pub fn from_str(contents: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = toml::from_str(contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                name: "OpenSOVD Gateway".to_string(),
            },
            auth: None,
            components: vec![],
        }
    }
}

impl Config {
    /// Returns the embedded/builtin configuration from config.toml
    pub fn builtin() -> Self {
        const EMBEDDED_CONFIG: &str = include_str!("../config.toml");
        Self::from_str(EMBEDDED_CONFIG).expect("Embedded config should be valid")
    }

    pub fn build_registry(self) -> Result<ComponentRegistry, Box<dyn std::error::Error>> {
        let mut registry_builder = ComponentRegistry::builder();

        #[cfg(feature = "config-entities")]
        {
            for component_cfg in self.components {
                // Determine the component ID
                let component_id = component_cfg.id.clone()
                    .unwrap_or_else(|| format!("component-{}", uuid::Uuid::new_v4()));

                // Create component based on whether it's remote or local
                let component = if let Some(remote_cfg) = component_cfg.remote {
                    // Remote component
                    let remote_resource = RemoteResource::new(remote_cfg.url)
                        .with_headers(remote_cfg.headers)
                        .with_timeout(remote_cfg.timeout_ms);
                    
                    Component::builder(component_id, component_cfg.name)
                        .with_resource(Box::new(remote_resource))
                        .build()
                } else {
                    // Local component with data
                    let mut data_resource = HashMapDataResource::new();

                    for data_cfg in component_cfg.data {
                        let metadata = DataItem {
                            id: data_cfg.id,
                            name: data_cfg.name,
                            category: data_cfg.category,
                            groups: data_cfg.groups,
                            tags: data_cfg.tags,
                        };
                        let data_value = DataValue {
                            value: data_cfg.value,
                            metadata,
                            read_only: !data_cfg.writable,
                        };
                        data_resource.insert(data_value);
                    }

                    let local_resource = LocalResource::new(Box::new(data_resource));
                    
                    Component::builder(component_id, component_cfg.name)
                        .with_resource(Box::new(local_resource))
                        .build()
                };

                registry_builder = registry_builder.with_component(component);
            }
        }

        #[cfg(not(feature = "config-entities"))]
        {
            if !self.components.is_empty() {
                tracing::debug!(
                    "Ignoring {} components (config-entities feature disabled)",
                    self.components.len()
                );
            }
        }

        Ok(registry_builder.build())
    }
}
