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
use crate::data::DataCategory;

/// Data item metadata with generic category support
/// 
/// # Type Parameters
/// * `V` - The vendor extension type (defaults to String)
/// 
/// # Examples
/// ```rust
/// // String-based vendor categories  
/// let item = DataItem {
///     id: "temperature".to_string(),
///     name: "Engine Temperature".to_string(),
///     translation_id: None,
///     category: DataCategory::Vendor("x-liebherr-engine".to_string()),
///     groups: vec!["engine".to_string()],
///     tags: vec!["temperature".to_string()],
/// };
/// 
/// // Typed vendor categories
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum LiebherrCategory { Engine, Hydraulics }
/// 
/// let typed_item = DataItem {
///     id: "pressure".to_string(),
///     name: "Hydraulic Pressure".to_string(),
///     translation_id: None,
///     category: DataCategory::Vendor(LiebherrCategory::Hydraulics),
///     groups: vec!["hydraulics".to_string()],
///     tags: vec!["pressure".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct DataItem<V = String>
where
    V: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug
{
    pub id: String,
    pub name: String,
    pub translation_id: Option<String>,
    pub category: DataCategory<V>,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
}

/// Convenient type aliases for common data item patterns
/// Standard ISO data items (no vendor extensions)
pub type StandardDataItem = DataItem<()>;

/// String-based vendor data items (most flexible)
pub type StringDataItem = DataItem<String>;

/// Serialization support for StringDataItem
impl Serialize for StringDataItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DataItem", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        if self.translation_id.is_some() {
            state.serialize_field("translation_id", &self.translation_id)?;
        }
        state.serialize_field("category", &self.category)?;
        if !self.groups.is_empty() {
            state.serialize_field("groups", &self.groups)?;
        }
        if !self.tags.is_empty() {
            state.serialize_field("tags", &self.tags)?;
        }
        state.end()
    }
}

/// Deserialization support for StringDataItem
impl<'de> Deserialize<'de> for StringDataItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Name,
            TranslationId,
            Category,
            Groups,
            Tags,
        }

        struct DataItemVisitor;

        impl<'de> serde::de::Visitor<'de> for DataItemVisitor {
            type Value = StringDataItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct DataItem")
            }

            fn visit_map<V>(self, mut map: V) -> Result<StringDataItem, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut translation_id = None;
                let mut category = None;
                let mut groups = None;
                let mut tags = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::TranslationId => {
                            if translation_id.is_some() {
                                return Err(serde::de::Error::duplicate_field("translation_id"));
                            }
                            translation_id = Some(map.next_value()?);
                        }
                        Field::Category => {
                            if category.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            category = Some(map.next_value()?);
                        }
                        Field::Groups => {
                            if groups.is_some() {
                                return Err(serde::de::Error::duplicate_field("groups"));
                            }
                            groups = Some(map.next_value()?);
                        }
                        Field::Tags => {
                            if tags.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let category = category.ok_or_else(|| serde::de::Error::missing_field("category"))?;
                let groups = groups.unwrap_or_default();
                let tags = tags.unwrap_or_default();

                Ok(StringDataItem {
                    id,
                    name,
                    translation_id,
                    category,
                    groups,
                    tags,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "name", "translation_id", "category", "groups", "tags"];
        deserializer.deserialize_struct("DataItem", FIELDS, DataItemVisitor)
    }
}