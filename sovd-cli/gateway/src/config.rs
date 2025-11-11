// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub listen: String,
    #[serde(default)]
    pub server_name: Vec<String>,
    #[serde(default = "default_base")]
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<SslConfig>,
}

fn default_base() -> String {
    "/".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_public_key_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SslConfig {
    /// Path to certificate file (PEM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,

    /// Path to private key file (PEM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Path to CA certificate file (PEM format) for peer verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cacert: Option<String>,

    /// Disable TLS certificate verification (insecure mode)
    #[serde(default)]
    pub insecure: bool,

    /// No peer certificate required (server mode only)
    #[serde(default)]
    pub no_peer_cert: bool,
}

#[cfg(feature = "openssl")]
impl From<&libsovd::SslArgs> for SslConfig {
    fn from(args: &libsovd::SslArgs) -> Self {
        Self {
            cert: args.cert.clone(),
            key: args.key.clone(),
            cacert: args.cacert.clone(),
            insecure: args.insecure,
            no_peer_cert: args.no_peer_cert,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_str(&contents)
    }

    pub fn from_str(contents: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Self = toml::from_str(contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: vec![ServerConfig {
                name: "SOVD Server".to_string(),
                listen: "127.0.0.1:9000".to_string(),
                server_name: vec![],
                base: "/sovd".to_string(),
                auth: None,
                ssl: None,
            }],
        }
    }
}
