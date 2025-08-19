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

use serde::{Serialize, de::DeserializeOwned};

/// Data resource trait for key-value storage with serializable values
pub trait Data: Send + Sync + 'static {
    type Filter: DeserializeOwned;
    type ReadValue: Serialize;
    type WriteValue: DeserializeOwned;
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    /// Read a value by key
    fn read(&self, key: &str) -> Option<Self::ReadValue>;

    /// Write a value with a key
    fn write(&mut self, key: &str, value: Self::WriteValue);

    /// Query data with an optional filter, returns an iterator
    fn query(&self, filter: Option<Self::Filter>) -> Self::Iter;
}
