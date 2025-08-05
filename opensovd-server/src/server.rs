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

use futures_core::future::BoxFuture;
use http::Uri;
use sovd::models::version::VendorInfo;

use crate::error::{ServerError, ServerResult};
use std::future::Future;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;

#[cfg(feature = "openssl")]
use openssl::ssl::SslAcceptorBuilder;

/// Default base URI for the OpenSOVD server
pub const DEFAULT_BASE_URI: &str = "http://localhost:9000/";

/// Server configuration containing binding and shutdown information.
pub struct ServerConfig<T = VendorInfo> {
    /// The socket configuration for the server
    pub(crate) socket: Option<Socket>,
    /// Optional shutdown signal future
    pub(crate) shutdown: Option<BoxFuture<'static, ()>>,
    /// Vendor information for the server
    pub(crate) vendor_info: Option<T>,
    /// Base URI for the server
    pub(crate) base_uri: Uri,
}

/// Socket configuration for the HTTP server.
pub(crate) enum Socket {
    /// Listen to a TCP socket
    TcpListener(TcpListener),
    /// Listen to a secure TCP socket
    #[cfg(feature = "openssl")]
    SecureTcpListener(TcpListener, SslAcceptorBuilder),
    /// Listen to a Unix domain socket
    #[cfg(unix)]
    UnixSocket(UnixListener),
}

impl std::fmt::Debug for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Socket::TcpListener(listener) => f.debug_tuple("TcpListener").field(listener).finish(),
            #[cfg(feature = "openssl")]
            Socket::SecureTcpListener(listener, _) => {
                // SslAcceptorBuilder doesn't implement Debug, so we just show the listener
                f.debug_tuple("SecureTcpListener")
                    .field(listener)
                    .field(&"<SslAcceptorBuilder>")
                    .finish()
            }
            #[cfg(unix)]
            Socket::UnixSocket(listener) => f.debug_tuple("UnixSocket").field(listener).finish(),
        }
    }
}

impl Default for ServerConfig<VendorInfo> {
    /// Creates a default server configuration.
    fn default() -> Self {
        Self {
            socket: None,
            shutdown: None,
            vendor_info: Some(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "OpenSOVD".to_string(),
            }),
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        }
    }
}

impl ServerConfig {
    /// Creates a new builder for ServerConfig with default VendorInfo type.
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }

    /// Creates a new builder for ServerConfig with custom vendor info type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    /// use opensovd_server::ServerConfig;
    ///
    /// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    /// struct CustomVendorInfo {
    ///     vendor: String,
    /// }
    ///
    /// let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    /// let vendor_info = CustomVendorInfo {
    ///     vendor: "My Company".to_string(),
    /// };
    /// let config = ServerConfig::builder_with_vendor_type::<CustomVendorInfo>()
    ///     .listen(listener)
    ///     .vendor_info(vendor_info)
    ///     .base_uri("https://api.example.com/sovd")
    ///     .unwrap()
    ///     .build();
    /// ```
    pub fn builder_with_vendor_type<T>() -> ServerConfigBuilder<T> {
        ServerConfigBuilder::new()
    }
}

/// Builder for ServerConfig that allows fluent configuration.
pub struct ServerConfigBuilder<T = VendorInfo> {
    socket: Option<Socket>,
    shutdown: Option<BoxFuture<'static, ()>>,
    vendor_info: Option<T>,
    base_uri: Option<Uri>,
}

impl<T> ServerConfigBuilder<T> {
    /// Creates a new ServerConfigBuilder.
    pub fn new() -> Self {
        Self {
            socket: None,
            shutdown: None,
            vendor_info: None,
            base_uri: None,
        }
    }

    /// Listen on a TCP socket using a TcpListener.
    pub fn listen(mut self, listener: TcpListener) -> Self {
        self.socket = Some(Socket::TcpListener(listener));
        self
    }

    /// Listen on a TCP socket using a TcpListener.
    #[cfg(feature = "openssl")]
    pub fn listen_openssl(mut self, listener: TcpListener, ssl_acceptor_builder: SslAcceptorBuilder) -> Self {
        self.socket = Some(Socket::SecureTcpListener(listener, ssl_acceptor_builder));
        self
    }

    /// Listen on a Unix domain socket using a UnixListener.
    #[cfg(unix)]
    pub fn listen_uds(mut self, listener: UnixListener) -> Self {
        self.socket = Some(Socket::UnixSocket(listener));
        self
    }

    /// Set the shutdown signal for the server.
    pub fn shutdown<Fut>(mut self, shutdown: Fut) -> Self
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.shutdown = Some(Box::pin(shutdown));
        self
    }

    /// Set the vendor information for the server.
    pub fn vendor_info(mut self, vendor_info: T) -> Self {
        self.vendor_info = Some(vendor_info);
        self
    }

    /// Set the base URI for the server from a Uri.
    pub fn base_uri<U>(mut self, base_uri: U) -> ServerResult<Self>
    where
        U: TryInto<Uri>,
        ServerError: From<U::Error>,
    {
        self.base_uri = Some(base_uri.try_into().map_err(ServerError::from)?);
        Ok(self)
    }

    /// Builds the ServerConfig.
    ///
    /// # Panics
    ///
    /// Panics if vendor_info has not been set.
    pub fn build(self) -> ServerConfig<T> {
        ServerConfig {
            socket: self.socket,
            shutdown: self.shutdown,
            vendor_info: self.vendor_info,
            base_uri: self.base_uri.unwrap_or_else(|| {
                DEFAULT_BASE_URI
                    .parse::<Uri>()
                    .expect("DEFAULT_BASE_URI should be valid")
            }),
        }
    }
}

impl Default for ServerConfigBuilder<VendorInfo> {
    fn default() -> Self {
        Self {
            socket: None,
            shutdown: None,
            vendor_info: Some(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "OpenSOVD".to_string(),
            }),
            base_uri: Some(
                DEFAULT_BASE_URI
                    .parse::<Uri>()
                    .expect("DEFAULT_BASE_URI should be valid"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_server_config() {
        let config = ServerConfig::default();
        assert!(config.socket.is_none());
        assert!(config.vendor_info.is_some());
        let vendor_info = config.vendor_info.unwrap();
        assert_eq!(vendor_info.name, "OpenSOVD");
        assert_eq!(vendor_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.base_uri.to_string(), DEFAULT_BASE_URI);
        assert!(config.shutdown.is_none());
    }

    #[test]
    fn test_server_config_builder() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "TestServer".to_string(),
        };
        let config = ServerConfig::builder()
            .listen(listener)
            .vendor_info(vendor_info.clone())
            .base_uri("http://test.example.com")
            .unwrap()
            .build();

        assert!(config.socket.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
        assert_eq!(config.base_uri.to_string(), "http://test.example.com/");
    }

    #[test]
    fn test_server_with_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "TestServer".to_string(),
        };

        let shutdown_future = async {};
        let config = ServerConfig::builder()
            .listen(listener)
            .vendor_info(vendor_info)
            .shutdown(shutdown_future)
            .build();

        assert!(config.socket.is_some());
        assert!(config.shutdown.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
    }
}
