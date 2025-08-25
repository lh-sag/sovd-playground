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

pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

pub const ENABLED_FEATURES: &[&str] = &[
    #[cfg(feature = "ui")]
    "ui",
    #[cfg(feature = "openssl")]
    "openssl",
    #[cfg(feature = "config")]
    "config",
];
