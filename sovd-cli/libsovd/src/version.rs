// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

pub const ENABLED_FEATURES: &[&str] = &[
    #[cfg(feature = "ui")]
    "ui",
    #[cfg(feature = "openssl")]
    "openssl",
    #[cfg(feature = "jsonschema-schemars")]
    "jsonschema-schemars",
];
