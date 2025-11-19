// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Reusable mock components for examples

use async_trait::async_trait;
use serde_json::json;
use sovd_diagnostic::{
    Entity,
    data::{DataCategory, DataCategoryInformation, DataError, DataService, DataValue, ValueGroup, ValueMetaData},
};

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,actix_server=warn,actix_web=warn")),
        )
        .init();
}

/// Simple engine component
pub struct Engine;

impl Entity for Engine {
    fn id(&self) -> &str {
        "engine"
    }

    fn name(&self) -> &str {
        "Engine Control Unit"
    }

    fn tags(&self) -> &[String] {
        &[]
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }
}

/// Simple read-only data service for engine
pub struct EngineData;

#[async_trait]
impl DataService for EngineData {
    async fn list(
        &self,
        _entity_id: &str,
        _categories: Vec<DataCategory>,
        _groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError> {
        Ok(vec![ValueMetaData {
            id: "rpm".to_string(),
            name: "Engine RPM".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec![],
            tags: vec![],
        }])
    }

    async fn list_categories(&self, _entity_id: &str) -> Result<Vec<DataCategoryInformation>, DataError> {
        Ok(vec![DataCategoryInformation {
            item: DataCategory::CurrentData,
            category_translation_id: None,
        }])
    }

    async fn list_groups(
        &self,
        _entity_id: &str,
        _category: Option<DataCategory>,
    ) -> Result<Vec<ValueGroup>, DataError> {
        Ok(vec![])
    }

    async fn read(&self, _entity_id: &str, data_id: &str) -> Result<DataValue, DataError> {
        if data_id == "rpm" {
            Ok(DataValue {
                id: "rpm".to_string(),
                value: json!(850),
                errors: vec![],
            })
        } else {
            Err(DataError::not_found(data_id))
        }
    }

    async fn write(&self, _entity_id: &str, data_id: &str, _value: serde_json::Value) -> Result<(), DataError> {
        // Read-only - reject all writes
        Err(DataError::not_found(data_id))
    }
}
