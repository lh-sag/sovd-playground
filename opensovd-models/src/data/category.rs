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
/// // Standard categories (no vendor extensions)
/// let standard = DataCategory::<()>::CurrentData;
/// 
/// // String-based vendor categories
/// let string_vendor = DataCategory::Vendor("x-liebherr-hydraulics".to_string());
/// 
/// // Typed vendor categories
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum MyVendorCategory { Hydraulics, Engine }
/// let typed_vendor = DataCategory::Vendor(MyVendorCategory::Hydraulics);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub enum DataCategory<V = String> 
where
    V: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug
{
    /// Identification data - static information about the entity
    IdentData,
    /// Current data - live/real-time values
    CurrentData,
    /// Stored data - historical/logged values
    StoredData,
    /// System information - diagnostic/status data
    SysInfo,
    /// Vendor-specific category extension
    Vendor(V),
}

/// Convenient type aliases for common usage patterns
/// Standard ISO categories only (no vendor extensions)
pub type StandardDataCategory = DataCategory<()>;

/// String-based vendor categories (most flexible)
pub type StringDataCategory = DataCategory<String>;

/// Serialization support for StringDataCategory
impl Serialize for StringDataCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DataCategory::IdentData => serializer.serialize_str("identData"),
            DataCategory::CurrentData => serializer.serialize_str("currentData"),
            DataCategory::StoredData => serializer.serialize_str("storedData"),
            DataCategory::SysInfo => serializer.serialize_str("sysInfo"),
            DataCategory::Vendor(vendor) => serializer.serialize_str(vendor),
        }
    }
}

/// Deserialization support for StringDataCategory
impl<'de> Deserialize<'de> for StringDataCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "identData" => Ok(DataCategory::IdentData),
            "currentData" => Ok(DataCategory::CurrentData),
            "storedData" => Ok(DataCategory::StoredData),
            "sysInfo" => Ok(DataCategory::SysInfo),
            vendor if vendor.starts_with("x-") => {
                StringDataCategory::new_string_vendor(vendor).map_err(serde::de::Error::custom)
            }
            _ => Err(serde::de::Error::custom(format!("Unknown data category: {}", s))),
        }
    }
}

impl<V> DataCategory<V> 
where
    V: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug
{
    /// Checks if this is a standard ISO category
    pub fn is_standard(&self) -> bool {
        matches!(self, DataCategory::IdentData | DataCategory::CurrentData | 
                      DataCategory::StoredData | DataCategory::SysInfo)
    }

    /// Checks if this is a vendor category
    pub fn is_vendor(&self) -> bool {
        matches!(self, DataCategory::Vendor(_))
    }

    /// Gets a reference to the vendor extension if this is a vendor category
    pub fn vendor(&self) -> Option<&V> {
        match self {
            DataCategory::Vendor(v) => Some(v),
            _ => None,
        }
    }
}

impl DataCategory<String> {
    /// Creates a new string-based vendor category with validation
    /// 
    /// # Arguments
    /// * `category` - The vendor category string (should start with "x-")
    /// 
    /// # Returns
    /// * `Ok(DataCategory::Vendor(...))` if valid
    /// * `Err(...)` if invalid format
    pub fn new_string_vendor(category: impl Into<String>) -> Result<Self, DataCategoryError> {
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
    if extension.is_empty() || extension.len() > 32 {
        return false;
    }
    
    if extension.ends_with('-') || extension.contains("--") {
        return false;
    }
    
    extension.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Error type for data category validation
#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum DataCategoryError {
    /// Invalid vendor category format
    #[display("Invalid vendor category format: '{}'. Must start with 'x-' and contain only lowercase letters, digits, and hyphens", _0)]
    InvalidFormat(#[error(ignore)] String),
}