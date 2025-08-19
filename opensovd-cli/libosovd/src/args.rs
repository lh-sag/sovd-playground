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

/// Common SSL/TLS arguments for OpenSOVD binaries supporting mTLS
#[cfg(feature = "openssl")]
#[derive(Parser, Clone, Debug, Default)]
pub struct SslArgs {
    /// Path to certificate file (PEM format)
    #[arg(long, help_heading = "Certificate Options")]
    pub cert: Option<String>,

    /// Path to private key file (PEM format)
    #[arg(long, help_heading = "Certificate Options")]
    pub key: Option<String>,

    /// Path to CA certificate file (PEM format) for peer verification
    #[arg(long, help_heading = "Certificate Options")]
    pub cacert: Option<String>,

    /// Disable TLS certificate verification (insecure mode)
    #[arg(long, help_heading = "Certificate Options")]
    pub insecure: bool,

    /// No peer certificate required (server mode only)
    #[arg(long = "no-peer-cert", help_heading = "Certificate Options")]
    pub no_peer_cert: bool,
}

#[cfg(feature = "openssl")]
impl SslArgs {
    /// Check if any SSL options are configured
    pub fn is_configured(&self) -> bool {
        self.cert.is_some() || self.key.is_some() || self.cacert.is_some() || self.insecure
    }

    /// Validate that both cert and key are provided together if either is specified
    pub fn validate_cert_key_pair(&self) -> Result<(), String> {
        match (&self.cert, &self.key) {
            (Some(_), Some(_)) | (None, None) => Ok(()),
            (Some(_), None) => Err("Both --cert and --key must be provided together".into()),
            (None, Some(_)) => Err("Both --cert and --key must be provided together".into()),
        }
    }
}
