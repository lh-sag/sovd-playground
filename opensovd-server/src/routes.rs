//
// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
//

pub(crate) mod data;
pub(crate) mod entity;
pub(crate) mod metrics;
pub(crate) mod proxy;
#[cfg(feature = "ui")]
pub(crate) mod ui;
pub(crate) mod version;

#[derive(Debug, Clone)]
pub(crate) struct BaseUri(pub String);
