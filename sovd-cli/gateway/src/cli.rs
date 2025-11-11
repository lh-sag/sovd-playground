// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use clap::Parser;
pub use libsovd::version::VERSION;

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(about = "SOVD gateway")]
#[command(version = VERSION)]
#[command(long_about = "SOVD Gateway

A SOVD server that can bind to HTTP, HTTPS, or Unix socket endpoints.
For multiple listeners, use a configuration file with --config.")]
#[command(after_help = "EXAMPLES:
  # HTTP listener
  sovd-gateway --url http://127.0.0.1:8080/sovd

  # HTTPS listener with mTLS
  sovd-gateway --url https://127.0.0.1:8443/sovd \\
    --cert server.pem --key server.key --cacert ca.pem

  # HTTPS with insecure mode (no client cert required)
  sovd-gateway --url https://127.0.0.1:8443/sovd \\
    --cert server.pem --key server.key --insecure --no-peer-cert

  # Unix domain socket
  sovd-gateway --url http://localhost/sovd --unix-socket /var/run/sovd.sock

  # Abstract Unix socket (Linux only)
  sovd-gateway --url http://localhost/sovd --unix-socket @sovd")]
pub struct Args {
    /// Base URL for SOVD server.
    ///
    /// Supported schemes: http://, https://
    #[arg(short = 'u', long = "url")]
    pub url: Option<String>,

    /// Path to Unix domain socket. Use @name for abstract sockets (Linux only).
    /// Requires --url to specify the base path.
    #[cfg(unix)]
    #[arg(long = "unix-socket", help_heading = "Listeners")]
    pub unix_socket: Option<String>,

    /// Path to configuration file.
    #[arg(short = 'c', long = "config")]
    pub config: Option<std::path::PathBuf>,

    /// Path to RSA public key file (PEM format) for JWT authentication.
    /// Can also be set via `SOVDLAB_AUTH_JWT` environment variable.
    #[arg(long = "auth-jwt", env = "SOVDLAB_AUTH_JWT", help_heading = "Authentication")]
    pub auth_jwt: Option<String>,

    #[cfg(feature = "openssl")]
    #[command(flatten)]
    pub ssl: libsovd::SslArgs,
}
