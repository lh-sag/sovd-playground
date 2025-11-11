// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use clap::Parser;

/// Common SSL/TLS arguments for SOVD binaries supporting mTLS
#[cfg(feature = "openssl")]
#[derive(Parser, Clone, Debug, Default)]
pub struct SslArgs {
    /// Path to certificate file (PEM format)
    #[arg(long, help_heading = "Certificate Options", requires = "key")]
    pub cert: Option<String>,

    /// Path to private key file (PEM format)
    #[arg(long, help_heading = "Certificate Options", requires = "cert")]
    pub key: Option<String>,

    /// Path to CA certificate file (PEM format) for peer verification.
    /// Required when using mTLS unless --insecure is specified.
    #[arg(long, help_heading = "Certificate Options")]
    pub cacert: Option<String>,

    /// Disable TLS certificate verification (insecure mode)
    #[arg(long, help_heading = "Certificate Options")]
    pub insecure: bool,

    /// No peer certificate required (server mode only).
    /// Still requires --cacert for optional client certificate verification unless --insecure is used.
    #[arg(long = "no-peer-cert", help_heading = "Certificate Options")]
    pub no_peer_cert: bool,
}

#[cfg(feature = "openssl")]
impl SslArgs {
    /// Check if any SSL options are configured
    #[must_use]
    pub const fn is_configured(&self) -> bool {
        self.cert.is_some() || self.key.is_some() || self.cacert.is_some() || self.insecure
    }

    /// Validate SSL configuration after parsing.
    /// This handles constraints that clap's declarative validation can't express.
    ///
    /// # Errors
    /// Returns a clap Error if validation fails
    pub fn validate(&self) -> Result<(), clap::Error> {
        // Only validate if SSL is actually configured
        if !self.is_configured() {
            return Ok(());
        }

        // If peer verification is enabled (not insecure) and SSL is used, cacert is required
        if !self.insecure && self.cert.is_some() && self.cacert.is_none() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::MissingRequiredArgument,
                "CA certificate (--cacert) is required when peer verification is enabled. \
                 Use --cacert to provide a CA certificate, or --insecure to disable verification.\n",
            ));
        }

        Ok(())
    }
}
