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
    /// Server URL (http:// or https://)
    #[arg(short = 'u', long = "url")]
    pub url: Option<String>,

    /// Unix socket path (@name for abstract)
    #[cfg(unix)]
    #[arg(long = "unix-socket", help_heading = "Listeners")]
    pub unix_socket: Option<String>,

    /// Config file path
    #[arg(short = 'c', long = "config")]
    pub config: Option<std::path::PathBuf>,

    /// RSA public key for JWT auth
    #[arg(long = "auth-jwt", help_heading = "Authentication")]
    pub auth_jwt: Option<String>,

    #[cfg(feature = "openssl")]
    #[command(flatten)]
    pub ssl: libsovd::SslArgs,
}
