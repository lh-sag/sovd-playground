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

use std::future::Future;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;

use futures_core::future::BoxFuture;
use opensovd_models::version::VendorInfo;
#[cfg(feature = "openssl")]
use openssl::ssl::SslAcceptorBuilder;

/// Server configuration containing binding and shutdown information.
pub struct ServerConfig<T = VendorInfo> {
    /// The listener configuration for the server
    pub(crate) listener: Option<Listener>,
    /// Optional shutdown signal future
    pub(crate) shutdown: Option<BoxFuture<'static, ()>>,
    /// Vendor information for the server
    pub(crate) vendor_info: Option<T>,
    /// URI path for the server
    pub(crate) uri_path: String,
}

/// Listener configuration for the HTTP server.
pub(crate) enum Listener {
    /// Listen to a TCP socket
    Tcp(TcpListener),
    /// Listen to a secure TCP socket
    #[cfg(feature = "openssl")]
    SecureTcp(TcpListener, SslAcceptorBuilder),
    /// Listen to a Unix domain socket
    #[cfg(unix)]
    Unix(UnixListener),
}

impl std::fmt::Debug for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Listener::Tcp(listener) => f.debug_tuple("Tcp").field(listener).finish(),
            #[cfg(feature = "openssl")]
            Listener::SecureTcp(listener, _) => {
                // SslAcceptorBuilder doesn't implement Debug, so we just show the listener
                f.debug_tuple("SecureTcp")
                    .field(listener)
                    .field(&"<SslAcceptorBuilder>")
                    .finish()
            }
            #[cfg(unix)]
            Listener::Unix(listener) => f.debug_tuple("Unix").field(listener).finish(),
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
    ///     .uri_path("/sovd")
    ///     .build();
    /// ```
    pub fn builder_with_vendor_type<T>() -> ServerConfigBuilder<T> {
        ServerConfigBuilder::new()
    }
}

/// Builder for ServerConfig that allows fluent configuration.
pub struct ServerConfigBuilder<T = VendorInfo> {
    listener: Option<Listener>,
    shutdown: Option<BoxFuture<'static, ()>>,
    vendor_info: Option<T>,
    uri_path: Option<String>,
}

impl<T> ServerConfigBuilder<T> {
    /// Creates a new ServerConfigBuilder.
    pub fn new() -> Self {
        Self {
            listener: None,
            shutdown: None,
            vendor_info: None,
            uri_path: None,
        }
    }

    /// Listen on a TCP socket using a TcpListener.
    pub fn listen(mut self, listener: TcpListener) -> Self {
        self.listener = Some(Listener::Tcp(listener));
        self
    }

    /// Listen on a TCP socket using a TcpListener.
    #[cfg(feature = "openssl")]
    pub fn listen_openssl(mut self, listener: TcpListener, ssl_acceptor_builder: SslAcceptorBuilder) -> Self {
        self.listener = Some(Listener::SecureTcp(listener, ssl_acceptor_builder));
        self
    }

    /// Listen on a Unix domain socket using a UnixListener.
    #[cfg(unix)]
    pub fn listen_uds(mut self, listener: UnixListener) -> Self {
        self.listener = Some(Listener::Unix(listener));
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

    /// Set the URI path for the server from a string.
    pub fn uri_path<U>(mut self, uri_path: U) -> Self
    where
        U: Into<String>,
    {
        self.uri_path = Some(uri_path.into());
        self
    }

    /// Builds the ServerConfig.
    ///
    /// # Panics
    ///
    /// Panics if vendor_info has not been set.
    pub fn build(self) -> ServerConfig<T> {
        ServerConfig {
            listener: self.listener,
            shutdown: self.shutdown,
            vendor_info: self.vendor_info,
            uri_path: self.uri_path.unwrap_or_else(|| "/".to_string()),
        }
    }
}

impl Default for ServerConfigBuilder<VendorInfo> {
    fn default() -> Self {
        Self {
            listener: None,
            shutdown: None,
            vendor_info: Some(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "OpenSOVD".to_string(),
            }),
            uri_path: Some("/".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .uri_path("/test")
            .build();

        assert!(config.listener.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
        assert_eq!(config.uri_path, "/test");
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

        assert!(config.listener.is_some());
        assert!(config.shutdown.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
    }
}
