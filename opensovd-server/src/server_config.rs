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
use std::sync::Arc;

use derive_more::{Debug, Display};
use futures_core::future::BoxFuture;
use opensovd_diagnostic::registry::ComponentRegistry;
use opensovd_models::version::VendorInfo;
#[cfg(feature = "openssl")]
use openssl::ssl::SslAcceptorBuilder;

/// Authentication configuration for the server
#[derive(Debug, Clone)]
pub struct AuthInfo {
    /// PEM-encoded public key for JWT validation
    pub public_key_pem: Vec<u8>,
}

/// Server configuration containing binding and shutdown information.
pub struct ServerConfig<T = VendorInfo> {
    /// The listener configurations for the server
    pub(crate) listeners: Vec<Listener>,
    /// Optional shutdown signal future
    pub(crate) shutdown: Option<BoxFuture<'static, ()>>,
    /// Vendor information for the server
    pub(crate) vendor_info: Option<T>,
    /// URI path for the server
    pub(crate) uri_path: String,
    /// Component registry for the server
    pub(crate) registry: Arc<ComponentRegistry>,
    /// Optional authentication configuration
    pub(crate) auth: Option<AuthInfo>,
}

/// Error types for the ServerConfigBuilder.
#[derive(Debug, Display)]
pub enum ServerConfigError {
    /// No listeners were configured for the server.
    #[display("No listeners configured. Please configure at least one listener.")]
    NoListenersConfigured,
}

impl std::error::Error for ServerConfigError {}

/// Listener configuration for the HTTP server.
#[derive(Debug)]
pub(crate) enum Listener {
    /// Listen to a TCP socket
    Tcp(TcpListener),
    /// Listen to a secure TCP socket
    #[cfg(feature = "openssl")]
    SecureTcp(TcpListener, #[debug(skip)] SslAcceptorBuilder),
    /// Listen to a Unix domain socket
    #[cfg(unix)]
    Unix(UnixListener),
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
///
/// # Example
///
/// ```rust
/// use std::net::TcpListener;
/// use opensovd_server::ServerConfig;
/// use opensovd_models::version::VendorInfo;
/// # use std::sync::Arc;
/// # use opensovd_diagnostic::registry::ComponentRegistry;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let http_listener = TcpListener::bind("127.0.0.1:8080")?;
/// let vendor_info = VendorInfo {
///     version: "1.0.0".to_string(),
///     name: "MyServer".to_string(),
/// };
///
/// let config = ServerConfig::builder()
///     .listen(http_listener)
///     .vendor_info(vendor_info)
///     .uri_path("/api")
///     .registry(Arc::new(ComponentRegistry::new()))
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # HTTPS Configuration
///
/// For HTTPS support, use the `listen_openssl()` method to add HTTPS listeners
/// with their SSL configuration:
///
/// ```rust
/// # #[cfg(feature = "openssl")] {
/// use std::net::TcpListener;
/// use opensovd_server::ServerConfig;
/// use openssl::ssl::{SslAcceptor, SslMethod, SslVerifyMode, SslFiletype};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let http_listener = TcpListener::bind("127.0.0.1:8080")?;
/// let https_listener = TcpListener::bind("127.0.0.1:8443")?;
///
/// // Configure SSL with mTLS
/// let mut ssl_builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server())?;
/// ssl_builder.set_private_key_file("server.key", SslFiletype::PEM)?;
/// ssl_builder.set_certificate_chain_file("server.pem")?;
/// ssl_builder.set_ca_file("ca.pem")?;
/// ssl_builder.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
///
/// let config = ServerConfig::builder()
///     .listen(http_listener)                          // HTTP listener
///     .listen_openssl(https_listener, ssl_builder)   // HTTPS listener with SSL config
///     .build()?;
/// # Ok(())
/// # }
/// # }
/// ```
pub struct ServerConfigBuilder<T = VendorInfo> {
    listeners: Vec<Listener>,
    shutdown: Option<BoxFuture<'static, ()>>,
    vendor_info: Option<T>,
    uri_path: Option<String>,
    registry: Arc<ComponentRegistry>,
    auth: Option<AuthInfo>,
}

impl<T> ServerConfigBuilder<T> {
    /// Creates a new ServerConfigBuilder.
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            shutdown: None,
            vendor_info: None,
            uri_path: None,
            registry: Arc::new(ComponentRegistry::new()),
            auth: None,
        }
    }

    /// Add a TCP socket listener using a TcpListener.
    pub fn listen(mut self, listener: TcpListener) -> Self {
        self.listeners.push(Listener::Tcp(listener));
        self
    }

    /// Add a secure TCP socket listener using a TcpListener with explicit SSL configuration.
    #[cfg(feature = "openssl")]
    pub fn listen_openssl(mut self, listener: TcpListener, ssl_acceptor_builder: SslAcceptorBuilder) -> Self {
        self.listeners.push(Listener::SecureTcp(listener, ssl_acceptor_builder));
        self
    }

    /// Add a Unix domain socket listener using a UnixListener.
    #[cfg(unix)]
    pub fn listen_uds(mut self, listener: UnixListener) -> Self {
        self.listeners.push(Listener::Unix(listener));
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

    /// Set the component registry for the server.
    pub fn registry(mut self, registry: Arc<ComponentRegistry>) -> Self {
        self.registry = registry;
        self
    }

    /// Set the authentication configuration for the server.
    pub fn auth(mut self, auth: AuthInfo) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Builds the ServerConfig.
    ///
    /// # Errors
    ///
    /// Returns an error if no listeners are configured.
    ///
    /// # Panics
    ///
    /// Panics if vendor_info has not been set.
    pub fn build(self) -> Result<ServerConfig<T>, ServerConfigError> {
        if self.listeners.is_empty() {
            return Err(ServerConfigError::NoListenersConfigured);
        }

        Ok(ServerConfig {
            listeners: self.listeners,
            shutdown: self.shutdown,
            vendor_info: self.vendor_info,
            uri_path: self.uri_path.unwrap_or_else(|| "/".to_string()),
            registry: self.registry,
            auth: self.auth,
        })
    }
}

impl Default for ServerConfigBuilder<VendorInfo> {
    fn default() -> Self {
        Self {
            listeners: Vec::new(),
            shutdown: None,
            vendor_info: Some(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "OpenSOVD".to_string(),
            }),
            uri_path: Some("/".to_string()),
            registry: Arc::new(ComponentRegistry::new()),
            auth: None,
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
            .build()
            .unwrap();

        assert!(!config.listeners.is_empty());
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
            .build()
            .unwrap();

        assert!(!config.listeners.is_empty());
        assert!(config.shutdown.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
    }
    #[test]
    fn test_multiple_listeners_configuration() {
        let listener1 = TcpListener::bind("127.0.0.1:0").unwrap();
        let listener2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "MultiListenerServer".to_string(),
        };

        let config = ServerConfig::builder()
            .listen(listener1)
            .listen(listener2)
            .vendor_info(vendor_info)
            .build()
            .unwrap();

        assert_eq!(config.listeners.len(), 2);
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "MultiListenerServer");
        assert_eq!(vendor.version, "1.0.0");
    }

    #[test]
    #[cfg(feature = "openssl")]
    fn test_listen_openssl_direct() {
        use openssl::ssl::{SslAcceptor, SslMethod};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "DirectSSLServer".to_string(),
        };

        let ssl_builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server()).unwrap();

        let config = ServerConfig::builder()
            .listen_openssl(listener, ssl_builder)
            .vendor_info(vendor_info)
            .build()
            .unwrap();

        assert_eq!(config.listeners.len(), 1);

        // Verify it's an HTTPS listener
        match &config.listeners[0] {
            #[cfg(feature = "openssl")]
            Listener::SecureTcp(_, _) => {} // Expected
            _ => panic!("Expected SecureTcp listener"),
        }
    }

    #[test]
    fn test_empty_listeners_validation() {
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "TestServer".to_string(),
        };

        let result = ServerConfig::builder().vendor_info(vendor_info).build();

        assert!(result.is_err());
        match result {
            Err(ServerConfigError::NoListenersConfigured) => {}
            _ => panic!("Expected NoListenersConfigured error"),
        }
    }
}
