// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use std::future::Future;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::sync::Arc;

use derive_more::{Debug, Display};
use futures_core::future::BoxFuture;
#[cfg(feature = "openssl")]
use openssl::ssl::SslAcceptorBuilder;
use sovd_diagnostic::Diagnostic;
use sovd_models::version::VendorInfo;

/// Authentication configuration for the server
#[derive(Debug, Clone)]
pub struct AuthInfo {
    /// PEM-encoded public key for JWT validation
    pub public_key_pem: Vec<u8>,
}

/// Represents a complete server endpoint with listener, auth, and routing information
#[derive(Debug)]
pub struct Endpoint {
    /// The listener for this endpoint
    pub listener: Listener,
    /// Optional authentication configuration for this endpoint
    pub auth: Option<AuthInfo>,
    /// Hostname(s) for virtual host matching (empty = match all hosts)
    pub server_name: Vec<String>,
    /// Base URI path for this endpoint (e.g., "/sovd", "/api")
    pub base: String,
}

/// Server configuration containing binding and shutdown information.
pub struct ServerConfig<T = VendorInfo> {
    /// The endpoints for the server
    pub(crate) endpoints: Vec<Endpoint>,
    /// Optional shutdown signal future
    pub(crate) shutdown: Option<BoxFuture<'static, ()>>,
    /// Vendor information for the server
    pub(crate) vendor_info: Option<T>,
    /// Diagnostic instance for the server
    pub(crate) diagnostic: Arc<Diagnostic>,
}

/// Error types for the ServerConfigBuilder.
#[derive(Debug, Display)]
pub enum ServerConfigError {
    /// No endpoints were configured for the server.
    #[display("No endpoints configured. Please configure at least one endpoint.")]
    NoEndpointsConfigured,
}

impl std::error::Error for ServerConfigError {}

/// Listener configuration for the HTTP server.
#[derive(Debug)]
pub enum Listener {
    /// Listen to a TCP socket
    Tcp(TcpListener),
    /// Listen to a secure TCP socket
    #[cfg(feature = "openssl")]
    SecureTcp(TcpListener, #[debug(skip)] SslAcceptorBuilder),
    /// Listen to a Unix domain socket
    #[cfg(unix)]
    Unix(UnixListener),
}

impl Listener {
    /// Get the actual bound address of the listener.
    /// Returns `None` if the address cannot be determined.
    #[must_use]
    pub fn local_addr(&self) -> Option<String> {
        match self {
            Self::Tcp(tcp) => tcp.local_addr().ok().map(|addr| addr.to_string()),
            #[cfg(feature = "openssl")]
            Self::SecureTcp(tcp, _) => tcp.local_addr().ok().map(|addr| addr.to_string()),
            #[cfg(unix)]
            Self::Unix(unix) => unix
                .local_addr()
                .ok()
                .and_then(|addr| addr.as_pathname().map(|p| p.display().to_string())),
        }
    }
}

impl ServerConfig {
    /// Creates a new builder for ServerConfig with default VendorInfo type.
    /// This builder comes with pre-populated vendor information.
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }

    /// Creates a new builder for ServerConfig with a custom vendor info type.
    /// The vendor_info field starts as None and must be set explicitly.
    pub fn builder_with_vendor_type<T>() -> ServerConfigBuilder<T> {
        ServerConfigBuilder::new()
    }
}

pub struct ServerConfigBuilder<T = VendorInfo> {
    endpoints: Vec<Endpoint>,
    shutdown: Option<BoxFuture<'static, ()>>,
    vendor_info: Option<T>,
    diagnostic: Arc<Diagnostic>,
}

impl<T> ServerConfigBuilder<T> {
    /// Creates a new ServerConfigBuilder.
    pub fn new() -> Self {
        Self {
            endpoints: Vec::new(),
            shutdown: None,
            vendor_info: None,
            diagnostic: Arc::new(Diagnostic::empty()),
        }
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

    /// Set the diagnostic instance for the server.
    pub fn diagnostic(mut self, diagnostic: Arc<Diagnostic>) -> Self {
        self.diagnostic = diagnostic;
        self
    }

    /// Add an endpoint with listener, auth, and routing information.
    /// This bundles the configuration for a complete server endpoint.
    pub fn endpoint(
        mut self,
        listener: Listener,
        auth: Option<AuthInfo>,
        server_name: Vec<String>,
        base: String,
    ) -> Self {
        self.endpoints.push(Endpoint {
            listener,
            auth,
            server_name,
            base,
        });

        self
    }

    pub fn build(self) -> Result<ServerConfig<T>, ServerConfigError> {
        if self.endpoints.is_empty() {
            return Err(ServerConfigError::NoEndpointsConfigured);
        }

        Ok(ServerConfig {
            endpoints: self.endpoints,
            shutdown: self.shutdown,
            vendor_info: self.vendor_info,
            diagnostic: self.diagnostic,
        })
    }
}

// Specialized Default implementation for VendorInfo type with pre-populated vendor info
impl Default for ServerConfigBuilder<VendorInfo> {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
            shutdown: None,
            vendor_info: Some(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "SOVD".to_string(),
            }),
            diagnostic: Arc::new(Diagnostic::empty()),
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
            .endpoint(
                Listener::Tcp(listener),
                None,
                vec!["localhost".to_string()],
                "/test".to_string(),
            )
            .vendor_info(vendor_info.clone())
            .build()
            .unwrap();

        assert!(!config.endpoints.is_empty());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
        assert_eq!(config.endpoints[0].base, "/test");
        assert_eq!(config.endpoints[0].server_name, vec!["localhost"]);
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
            .endpoint(Listener::Tcp(listener), None, vec![], "/test".to_string())
            .vendor_info(vendor_info)
            .shutdown(shutdown_future)
            .build()
            .unwrap();

        assert!(!config.endpoints.is_empty());
        assert!(config.shutdown.is_some());
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "TestServer");
        assert_eq!(vendor.version, "1.0.0");
    }
    #[test]
    fn test_multiple_endpoints_configuration() {
        let listener1 = TcpListener::bind("127.0.0.1:0").unwrap();
        let listener2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "MultiEndpointServer".to_string(),
        };

        let config = ServerConfig::builder()
            .endpoint(Listener::Tcp(listener1), None, vec![], "/api1".to_string())
            .endpoint(Listener::Tcp(listener2), None, vec![], "/api2".to_string())
            .vendor_info(vendor_info)
            .build()
            .unwrap();

        assert_eq!(config.endpoints.len(), 2);
        assert!(config.vendor_info.is_some());
        let vendor = config.vendor_info.unwrap();
        assert_eq!(vendor.name, "MultiEndpointServer");
        assert_eq!(vendor.version, "1.0.0");
    }

    #[test]
    #[cfg(feature = "openssl")]
    fn test_endpoint_with_ssl() {
        use openssl::ssl::{SslAcceptor, SslMethod};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "DirectSSLServer".to_string(),
        };

        let ssl_builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server()).unwrap();

        let config = ServerConfig::builder()
            .endpoint(
                Listener::SecureTcp(listener, ssl_builder),
                None,
                vec!["localhost".to_string()],
                "/secure".to_string(),
            )
            .vendor_info(vendor_info)
            .build()
            .unwrap();

        assert_eq!(config.endpoints.len(), 1);

        // Verify it's an HTTPS listener
        match &config.endpoints[0].listener {
            #[cfg(feature = "openssl")]
            Listener::SecureTcp(_, _) => {} // Expected
            _ => panic!("Expected SecureTcp listener"),
        }
    }

    #[test]
    fn test_empty_endpoints_validation() {
        let vendor_info = VendorInfo {
            version: "1.0.0".to_string(),
            name: "TestServer".to_string(),
        };

        let result = ServerConfig::builder().vendor_info(vendor_info).build();

        assert!(result.is_err());
        match result {
            Err(ServerConfigError::NoEndpointsConfigured) => {}
            _ => panic!("Expected NoEndpointsConfigured error"),
        }
    }
}
