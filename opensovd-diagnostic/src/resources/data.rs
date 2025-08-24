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


// Re-export data types from opensovd-models
pub use opensovd_models::data::{
    DataCategory, DataError, DataErrorCode, DataItem,
    StandardDataCategory, StandardDataItem, StringDataCategory, StringDataItem,
};

/// ISO 17978-3 compliant data resource trait
///
/// This trait provides access to data resources within diagnostic entities.
/// Data resources contain key-value pairs where values are JSON and items
/// are categorized according to the ISO standard.
pub trait DataResource: Send + Sync + 'static {
    /// List all data items with optional filtering (string-based categories)
    ///
    /// # Arguments
    /// * `categories` - Filter by data categories (empty = all categories)
    /// * `groups` - Filter by groups (empty = all groups)
    ///
    /// # Returns
    /// Vector of data item metadata matching the filters
    fn list_data_items(&self, categories: &[StringDataCategory], groups: &[String]) -> Vec<StringDataItem>;

    /// Read a specific data value
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to read
    ///
    /// # Returns
    /// The data value as JSON, or a DataError if not found/accessible
    fn read_data(&self, data_id: &str) -> Result<serde_json::Value, DataError>;

    /// Write a specific data value
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to write
    /// * `value` - The new value as JSON
    ///
    /// # Returns
    /// Success or DataError if write failed
    fn write_data(&mut self, data_id: &str, value: serde_json::Value) -> Result<(), DataError>;

    /// Check if a data item exists
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item to check
    ///
    /// # Returns
    /// True if the data item exists, false otherwise
    fn has_data_item(&self, data_id: &str) -> bool;

    /// Get metadata for a specific data item
    ///
    /// # Arguments
    /// * `data_id` - The ID of the data item
    ///
    /// # Returns
    /// Data item metadata if found, None otherwise
    fn get_data_item(&self, data_id: &str) -> Option<StringDataItem>;
}

