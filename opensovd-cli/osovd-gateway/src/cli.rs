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
#[cfg(feature = "openssl")]
use libosovd::SslArgs;

pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');
const DEFAULT_BASE_URI: &str = "http://127.0.0.1:9000/opensovd";

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(about = "OpenSOVD server")]
#[command(version = VERSION)]
#[command(long_about = "OpenSOVD Gateway Server

This server can bind to multiple URLs simultaneously. Specify multiple --url arguments
to listen on different ports, interfaces, or socket types. HTTPS URLs require TLS
certificate configuration.")]
#[command(after_help = "EXAMPLES:
  # Single HTTP listener
  osovd-gateway --url http://127.0.0.1:8080/opensovd

  # Multiple HTTP listeners on different ports
  osovd-gateway --url http://127.0.0.1:8080/opensovd --url http://127.0.0.1:9000/opensovd

  # HTTP and HTTPS listeners with mTLS
  osovd-gateway --url http://127.0.0.1:8080/opensovd --url https://127.0.0.1:8443/opensovd \\
    --cert server.pem --key server.key --cacert ca.pem

  # HTTPS with insecure mode (no client cert required)
  osovd-gateway --url https://127.0.0.1:8443/opensovd \\
    --cert server.pem --key server.key --insecure --no-peer-cert

  # HTTP and Unix socket listeners
  osovd-gateway --url http://127.0.0.1:8080/opensovd --url unix:///tmp/opensovd.sock")]
pub struct Args {
    /// Base URL for OpenSOVD server. Specify multiple times for multiple listeners.
    ///
    /// Supported schemes: http://, https://, unix://, http+unix://
    #[arg(short = 'u', long = "url", action = clap::ArgAction::Append)]
    pub urls: Vec<String>,

    #[cfg(feature = "openssl")]
    #[command(flatten)]
    pub ssl: SslArgs,
}

impl Args {
    /// Gets the URLs, using the default URL if none were provided.
    pub fn get_urls(&self) -> Vec<String> {
        if self.urls.is_empty() {
            vec![DEFAULT_BASE_URI.to_string()]
        } else {
            self.urls.clone()
        }
    }
}
