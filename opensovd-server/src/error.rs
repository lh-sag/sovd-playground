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

use derive_more::{Display, Error as DeriveError, From};

#[derive(Debug, Display, From, DeriveError)]
pub enum Error {
    #[display("IO error: {}", _0)]
    Io(std::io::Error),

    #[display("Invalid URI: {}", _0)]
    InvalidUri(http::uri::InvalidUri),

    #[display("Bad configuration: {}", _0)]
    #[from(ignore)]
    #[error(ignore)]
    BadConfiguration(String),

    #[display("Authentication setup failed: {}", _0)]
    #[from(ignore)]
    #[error(ignore)]
    AuthSetupFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;

