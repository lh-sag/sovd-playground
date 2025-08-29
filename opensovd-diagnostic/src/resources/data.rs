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
//

use async_trait::async_trait;

/// Simplified data error for diagnostic operations
///
/// This error type includes common error cases needed
/// for data resource operations.
#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum DataError {
    /// Data item not found with the given ID
    #[display("Data item not found: {}", _0)]
    DataNotFound(#[error(ignore)] String),
    /// Data item is read-only and cannot be written
    #[display("Data item is read-only: {}", _0)]
    ReadOnly(#[error(ignore)] String),
    /// Access denied to the data item
    #[display("Access denied: {}", _0)]
    AccessDenied(#[error(ignore)] String),
    /// Invalid data format or type
    #[display("Invalid data: {}", _0)]
    InvalidData(#[error(ignore)] String),
    /// Internal error occurred
    #[display("Internal error: {}", _0)]
    InternalError(#[error(ignore)] String),
}

/// Data item metadata for list operations
///
/// This struct contains the essential metadata fields needed for listing
/// and filtering data items within a data resource.
#[derive(Debug, Clone, PartialEq)]
pub struct DataItem {
    /// Unique identifier for the data item
    pub id: String,
    /// Human-readable name of the data item
    pub name: String,
    /// Category classification as a string
    pub category: String,
    /// Groups this data item belongs to
    pub groups: Vec<String>,
    /// Tags for additional classification
    pub tags: Vec<String>,
}

/// ISO 17978-3 compliant data resource trait
///
/// This trait provides access to data resources within diagnostic entities.
/// Data resources contain key-value pairs where values are JSON and items
/// are categorized according to the ISO standard.
#[async_trait]
pub trait DataResource: Send + Sync + 'static {
    /// List all data items with optional filtering (string-based categories)
    ///
    /// # Arguments
    /// * `categories` - Filter by data categories (empty = all categories)
    /// * `groups` - Filter by groups (empty = all groups)
    ///
    /// # Returns
    /// Vector of data item metadata matching the filters
    async fn list_data_items(&self, categories: &[String], groups: &[String]) -> Vec<DataItem>;

    /// Read a specific data value
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to read
    ///
    /// # Returns
    /// The data value as JSON, or a DataError if not found/accessible
    async fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError>;

    /// Write a specific data value
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to write
    /// * `value` - The new value as JSON
    ///
    /// # Returns
    /// Success or DataError if write failed
    async fn write_data(&mut self, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;

    /// Check if a data item exists
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to check
    ///
    /// # Returns
    /// True if the data item exists, false otherwise
    async fn has_data_item(&self, data_id: &str) -> bool;

    /// Get metadata for a specific data item
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item
    ///
    /// # Returns
    /// Data item metadata if found, None otherwise
    async fn get_data_item(&self, data_id: &str) -> Option<DataItem>;
}
