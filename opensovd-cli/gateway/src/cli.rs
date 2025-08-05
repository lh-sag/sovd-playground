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

use clap::Parser;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');
const DEFAULT_BASE_URI: &str = "http://localhost:9000/opensovd";

#[cfg(feature = "openssl")]
#[derive(Parser)]
pub struct SslArgs {
    /// Use the certificate authorities ("CA") to verify the peers
    #[arg(long, help_heading = "Certificate Options")]
    pub cacert: Option<String>,

    /// Use the server certificate stored in file.
    #[arg(long, help_heading = "Certificate Options")]
    pub cert: Option<String>,

    /// Use the private key in file
    #[arg(long, help_heading = "Certificate Options")]
    pub key: Option<String>,

    /// No peer verification
    #[arg(long, help_heading = "Certificate Options")]
    pub insecure: bool,

    /// No peer certificate required
    #[arg(long = "no-peer-cert", help_heading = "Certificate Options")]
    pub no_peer_cert: bool,
}

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(about = "OpenSOVD server")]
#[command(version = VERSION)]
pub struct Args {
    /// Base URI of the OpenSOVD server
    #[arg(short = 'u', long, default_value = DEFAULT_BASE_URI)]
    pub uri: String,

    #[cfg(feature = "openssl")]
    #[command(flatten)]
    pub ssl: SslArgs,
}
