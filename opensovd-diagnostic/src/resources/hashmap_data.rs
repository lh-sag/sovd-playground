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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::Data;

/// Example implementation of Data resource using HashMap
#[derive(Clone)]
pub struct HashMapData<V: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync> {
    data: HashMap<String, V>,
}

impl<V: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync> HashMapData<V> {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Creates a new HashMapData from an existing HashMap
    ///
    /// This is a convenience method for creating a HashMapData instance
    /// from pre-built data, avoiding the need to create an empty instance
    /// and populate it using individual write operations.
    ///
    /// # Arguments
    ///
    /// * `data` - A HashMap containing the initial data for this resource
    ///
    /// # Returns
    ///
    /// A new HashMapData instance containing the provided data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use opensovd_diagnostic::resources::HashMapData;
    ///
    /// let data = HashMap::from([
    ///     ("temperature".to_string(), "85.5".to_string()),
    ///     ("pressure".to_string(), "14.7".to_string()),
    /// ]);
    ///
    /// let hashmap_data = HashMapData::from_hashmap(data);
    /// ```
    pub fn from_hashmap(data: HashMap<String, V>) -> Self {
        Self { data }
    }
}

impl<V: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> Data for HashMapData<V> {
    type Filter = serde_json::Value;
    type ReadValue = V;
    type WriteValue = V;
    type Item = (String, V);
    type Iter = std::vec::IntoIter<Self::Item>;

    fn read(&self, key: &str) -> Option<Self::ReadValue> {
        self.data.get(key).cloned()
    }

    fn write(&mut self, key: &str, value: Self::WriteValue) {
        self.data.insert(key.to_string(), value);
    }

    fn query(&self, _filter: Option<Self::Filter>) -> Self::Iter {
        // Simple implementation: return all data (filter can be applied in more complex implementations)
        self.data
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> Default for HashMapData<V> {
    fn default() -> Self {
        Self::new()
    }
}
