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

use opensovd_diagnostic::{Diagnostic, resources::data::DataItem};
use serde::{Deserialize, Serialize};

#[cfg(feature = "config")]
use crate::hashmap_data_resource::{HashMapDataResource, DataValue};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub components: Vec<ComponentConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub default_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentConfig {
    pub id: String,
    pub name: String,
    pub data: Vec<DataItemConfig>,
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
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn from_str(contents: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = toml::from_str(contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        const EMBEDDED_CONFIG: &str = include_str!("../config.toml");
        Self::from_str(EMBEDDED_CONFIG).expect("Embedded config should be valid")
    }
}

impl Config {
    /// Build diagnostic from configuration
    pub fn build_diagnostic(self) -> Result<Diagnostic, Box<dyn std::error::Error>> {
        let mut builder = Diagnostic::builder();

        for component_cfg in self.components {
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

            builder = builder
                .component(component_cfg.name)
                .id(component_cfg.id)
                .data_resource(data_resource)
                .add()?;
        }

        Ok(builder.build())
    }
}
