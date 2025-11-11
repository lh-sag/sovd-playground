// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
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
