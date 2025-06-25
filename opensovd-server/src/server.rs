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

/// Default base URI for the OpenSOVD server
pub const DEFAULT_BASE_URI: &str = "http://localhost:9000/opensovd";

/// Server configuration containing binding and shutdown information.
pub struct ServerConfig<T = VendorInfo> {
    /// The socket configuration for the server
    socket: Option<Socket>,
    /// Optional shutdown signal future
    pub shutdown: Option<BoxFuture<'static, ()>>,
    /// Vendor information for the server
    pub vendor_info: T,
    /// Base URI for the server
    pub base_uri: Uri,
}

/// Socket configuration for the HTTP server.
#[derive(Debug)]
pub(crate) enum Socket {
    /// Bind to a TCP socket using a TcpListener
    TcpListener(TcpListener),
    /// Bind to a Unix domain socket using a UnixListener
    #[cfg(unix)]
    UnixSocket(UnixListener),
}

impl Default for ServerConfig<VendorInfo> {
    /// Creates a default server configuration listening on `127.0.0.1:8080`.
    fn default() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        Self {
            socket: Some(Socket::TcpListener(listener)),
            shutdown: None,
            vendor_info: VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "OpenSOVD".to_string(),
            },
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        }
    }
}

impl<T> ServerConfig<T> {
    /// Returns a reference to the socket configuration.
    pub(crate) fn socket(&self) -> Option<&Socket> {
        self.socket.as_ref()
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
    ///     .listen_address(listener)
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
    pub fn listen_address(mut self, listener: TcpListener) -> Self {
        self.socket = Some(Socket::TcpListener(listener));
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
            vendor_info: self.vendor_info.expect("vendor_info must be set before building"),
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
    fn test_server_config_builder() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder().listen_address(listener).build();
        match config.socket() {
            Some(Socket::TcpListener(_)) => {}
            #[cfg(unix)]
            _ => panic!("Expected TcpListener"),
        }
        assert!(config.shutdown.is_none());

        #[cfg(target_os = "linux")]
        {
            use std::os::linux::net::SocketAddrExt;
            use std::os::unix::net::SocketAddr;

            // Use abstract Unix domain socket
            let socket_addr = SocketAddr::from_abstract_name("test_server_builder").unwrap();
            let listener = UnixListener::bind_addr(&socket_addr).unwrap();
            let config = ServerConfig::builder().listen_uds(listener).build();
            match config.socket() {
                Some(Socket::UnixSocket(_)) => {}
                _ => panic!("Expected UnixSocket"),
            }
            assert!(config.shutdown.is_none());
        }
    }

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        match config.socket() {
            Some(Socket::TcpListener(_)) => {}
            #[cfg(unix)]
            _ => panic!("Expected default to be TcpListener"),
        }
        assert!(config.shutdown.is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_shutdown_builder() {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixListener};

        // Use abstract Unix domain socket
        let socket_addr = SocketAddr::from_abstract_name("test_shutdown_builder").unwrap();
        let listener = UnixListener::bind_addr(&socket_addr).unwrap();
        let config = ServerConfig::builder()
            .listen_uds(listener)
            .shutdown(std::future::ready(()))
            .build();

        match config.socket() {
            Some(Socket::UnixSocket(_)) => {}
            _ => panic!("Expected UnixSocket"),
        }
        assert!(config.shutdown.is_some());
    }

    #[test]
    fn test_custom_vendor_info_type() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        struct CustomVendorInfo {
            version: String,
            company: String,
            license: String,
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let custom_vendor_info = CustomVendorInfo {
            version: "1.5.0".to_string(),
            company: "Custom Corp".to_string(),
            license: "MIT".to_string(),
        };

        let config = ServerConfig::builder_with_vendor_type::<CustomVendorInfo>()
            .listen_address(listener)
            .vendor_info(custom_vendor_info.clone())
            .build();

        assert_eq!(config.vendor_info.company, "Custom Corp");
        assert_eq!(config.vendor_info.license, "MIT");
        assert_eq!(config.vendor_info.version, "1.5.0");
    }

    #[test]
    #[cfg(feature = "http2")]
    fn test_http2_feature_enabled() {
        // This test verifies that the http2 feature compiles correctly
        // when enabled. The actual HTTP/2 functionality is provided by
        // the underlying actix-web framework.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder().listen_address(listener).build();

        // Verify that config can be created with http2 feature enabled
        match config.socket() {
            Some(Socket::TcpListener(_)) => {}
            #[cfg(unix)]
            _ => panic!("Expected TcpListener"),
        }
    }

    #[test]
    fn test_base_uri_configuration() {
        // Test default base_uri
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder().listen_address(listener).build();
        assert_eq!(config.base_uri.to_string(), DEFAULT_BASE_URI);

        // Test custom base_uri with string
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let custom_uri = "https://api.example.com/v2";
        let config = ServerConfig::builder()
            .listen_address(listener)
            .base_uri(custom_uri)
            .unwrap()
            .build();
        assert_eq!(config.base_uri.to_string(), custom_uri);

        // Test base_uri with Uri type
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let uri: Uri = "https://api.example.com/v3".parse().unwrap();
        let config = ServerConfig::builder()
            .listen_address(listener)
            .base_uri(uri.clone())
            .unwrap()
            .build();
        assert_eq!(config.base_uri, uri);

        // Test invalid URI
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let result = ServerConfig::builder()
            .listen_address(listener)
            .base_uri("not a valid uri");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_socket_configuration() {
        // Test that ServerConfig can be created without a socket
        let vendor_info = VendorInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            name: "Test Server".to_string(),
        };

        let config = ServerConfig::builder().vendor_info(vendor_info).build();

        // Should have no socket configured
        assert!(config.socket().is_none());
    }
}
