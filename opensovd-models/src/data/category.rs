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

use serde::{Deserialize, Serialize};

/// Data categories with support for vendor extensions
///
/// This generic enum allows both standard ISO 17978-3 categories and vendor-specific
/// extensions through the type parameter V.
///
/// # Type Parameters
/// * `V` - The vendor extension type (defaults to String for flexibility)
///
/// # Examples
/// ```rust
/// use opensovd_models::data::DataCategory;
///
/// // Standard categories
/// let standard = DataCategory::CurrentData;
///
/// // String-based vendor categories
/// let string_vendor = DataCategory::Vendor("x-liebherr-hydraulics".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub enum DataCategory {
    /// Identification data - static information about the entity
    #[display("identData")]
    #[serde(rename = "identData")]
    IdentData,
    /// Current data - live/real-time values
    #[display("currentData")]
    #[serde(rename = "currentData")]
    CurrentData,
    /// Stored data - historical/logged values
    #[display("storedData")]
    #[serde(rename = "storedData")]
    StoredData,
    /// System information - diagnostic/status data
    #[display("sysInfo")]
    #[serde(rename = "sysInfo")]
    SysInfo,
    /// Vendor-specific category extension
    #[display("{}", _0)]
    Vendor(String),
}

impl DataCategory {
    /// Creates a new string-based vendor category with validation
    ///
    /// # Arguments
    /// * `category` - The vendor category string (should start with "x-")
    ///
    /// # Returns
    /// * `Ok(DataCategory::Vendor(...))` if valid
    /// * `Err(...)` if invalid format
    pub fn new_vendor(category: impl Into<String>) -> Result<Self, DataCategoryError> {
        let category_str = category.into();
        if is_valid_vendor_category(&category_str) {
            Ok(DataCategory::Vendor(category_str))
        } else {
            Err(DataCategoryError::InvalidFormat(category_str))
        }
    }
}

/// Validates vendor category format: x-<extension>
///
/// Rules:
/// - Must start with "x-"
/// - Extension must be 1-32 characters
/// - Extension can contain: a-z, 0-9, hyphens (no consecutive hyphens)
/// - Must not end with hyphen
fn is_valid_vendor_category(category: &str) -> bool {
    if !category.starts_with("x-") {
        return false;
    }

    let extension = &category[2..];
    if extension.is_empty() {
        return false;
    }

    extension
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Error type for data category validation
#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum DataCategoryError {
    /// Invalid vendor category format
    #[display(
        "Invalid vendor category format: '{}'. Must start with 'x-' and contain only lowercase letters, digits, and hyphens",
        _0
    )]
    InvalidFormat(#[error(ignore)] String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_vendor_categories() {
        // Test basic valid vendor categories
        assert!(DataCategory::new_vendor("x-liebherr").is_ok());
        assert!(DataCategory::new_vendor("x-caterpillar-engine").is_ok());
        assert!(DataCategory::new_vendor("x-test123").is_ok());
        assert!(DataCategory::new_vendor("x-under_score").is_ok());

        // Test that valid vendor categories display correctly
        let vendor = DataCategory::new_vendor("x-test").unwrap();
        assert_eq!(format!("{vendor}"), "x-test");

        // Test various valid combinations
        assert!(DataCategory::new_vendor("x-a").is_ok());
        assert!(DataCategory::new_vendor("x-123").is_ok());
        assert!(DataCategory::new_vendor("x-test-with-hyphens").is_ok());
        assert!(DataCategory::new_vendor("x-mix3d_ch4rs").is_ok());
    }

    #[test]
    fn test_invalid_vendor_categories() {
        // Test missing prefix
        assert!(DataCategory::new_vendor("no-prefix").is_err());
        assert!(DataCategory::new_vendor("liebherr").is_err());

        // Test empty extension
        assert!(DataCategory::new_vendor("x-").is_err());

        // Test uppercase characters
        assert!(DataCategory::new_vendor("x-UPPER").is_err());
        assert!(DataCategory::new_vendor("x-Mixed").is_err());

        // Test invalid characters
        assert!(DataCategory::new_vendor("x-space here").is_err());
        assert!(DataCategory::new_vendor("x-special@").is_err());
        assert!(DataCategory::new_vendor("x-dot.here").is_err());
        assert!(DataCategory::new_vendor("x-slash/here").is_err());

        // Test wrong prefix
        assert!(DataCategory::new_vendor("y-wrong").is_err());
        assert!(DataCategory::new_vendor("X-wrong").is_err());

        // Test completely invalid formats
        assert!(DataCategory::new_vendor("").is_err());
        assert!(DataCategory::new_vendor("x").is_err());
        assert!(DataCategory::new_vendor("-test").is_err());
    }
}
