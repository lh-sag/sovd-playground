// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::entities::EntityId;

#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum DataError {
    #[display("Data item not found: {}", _0)]
    NotFound(#[error(ignore)] String),

    #[display("Data item is read-only: {}", _0)]
    ReadOnly(#[error(ignore)] String),

    #[display("Access denied: {}", _0)]
    AccessDenied(#[error(ignore)] String),

    #[display("Invalid data: {}", _0)]
    InvalidData(#[error(ignore)] String),

    #[display("Internal error: {}", _0)]
    Internal(#[error(ignore)] String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataCategory {
    IdentData,
    CurrentData,
    StoredData,
    SysInfo,
    Vendor(String),
}

/// Value metadata matching ISO 17978-3 Section 7.9.3
#[derive(Debug, Clone, PartialEq)]
pub struct ValueMetaData {
    pub id: String,
    pub name: String,
    pub category: DataCategory,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DataValue {
    pub id: String,
    pub value: serde_json::Value,
    pub errors: Vec<DataError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataCategoryInformation {
    pub item: DataCategory,
    pub category_translation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueGroup {
    pub id: String,
    pub category: DataCategory,
    pub category_translation_id: Option<String>,
    pub group: Option<String>,
    pub group_translation_id: Option<String>,
}

#[async_trait]
pub trait DataService: Send + Sync {
    async fn list(
        &self,
        entity: &EntityId,
        categories: Vec<DataCategory>,
        groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError>;

    async fn list_categories(&self, entity: &EntityId) -> Result<Vec<DataCategoryInformation>, DataError>;

    async fn list_groups(
        &self,
        entity: &EntityId,
        category: Option<DataCategory>,
    ) -> Result<Vec<ValueGroup>, DataError>;

    async fn read(&self, entity: &EntityId, data_id: &str) -> Result<DataValue, DataError>;

    async fn write(&self, entity: &EntityId, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;
}
