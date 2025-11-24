// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use sovd_models::data::{DataCategory, DataCategoryInformation, DataError, ReadValue, ValueGroup, ValueMetaData};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataWithSchema<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

impl<T> DataWithSchema<T> {
    pub fn new(data: T) -> Self {
        Self { data, schema: None }
    }

    pub fn with_schema(data: T, schema: serde_json::Value) -> Self {
        Self {
            data,
            schema: Some(schema),
        }
    }
}

#[async_trait]
pub trait DataService: Send + Sync {
    async fn list(
        &self,
        entity_id: &str,
        categories: Vec<DataCategory>,
        groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError>;

    async fn list_categories(&self, entity_id: &str) -> Result<Vec<DataCategoryInformation>, DataError>;

    async fn list_groups(&self, entity_id: &str, category: Option<DataCategory>) -> Result<Vec<ValueGroup>, DataError>;

    async fn read(&self, entity_id: &str, data_id: &str) -> Result<ReadValue, DataError>;

    async fn read_with_schema(&self, entity_id: &str, data_id: &str) -> Result<DataWithSchema<ReadValue>, DataError> {
        let value = self.read(entity_id, data_id).await?;
        #[cfg(feature = "jsonschema-schemars")]
        {
            let schema = schemars::schema_for_value!(&value.data);
            let schema_json = serde_json::to_value(&schema).unwrap_or_else(|_| serde_json::json!({}));
            Ok(DataWithSchema::with_schema(value, schema_json))
        }
        #[cfg(not(feature = "jsonschema-schemars"))]
        {
            Ok(DataWithSchema::new(value))
        }
    }

    async fn write(&self, entity_id: &str, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;
}
