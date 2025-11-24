// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
pub use sovd_models::data::{DataCategory, DataCategoryInformation, DataError, ReadValue, ValueGroup, ValueMetaData};

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

    async fn write(&self, entity_id: &str, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;
}
