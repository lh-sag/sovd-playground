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

use std::time::Duration;

use derive_more::{Display, Error};
use url::Url;

/// Configuration errors that can occur during client setup.
///
/// This enum represents errors that occur when configuring the OpenSOVD client,
/// such as invalid URLs or unsupported configurations.
///
/// # Examples
///
/// ```rust
/// use opensovd_client::{ClientConfig, ClientConfigError};
///
/// match ClientConfig::builder("not-a-url".to_string()).build() {
///     Err(ClientConfigError::InvalidUrl(e)) => {
///         println!("URL parsing failed: {}", e);
///     }
///     Err(ClientConfigError::MalformedUrl(msg)) => {
///         println!("URL validation failed: {}", msg);
///     }
///     Ok(_) => println!("Configuration is valid"),
/// }
/// ```
#[derive(Debug, Display, Error)]
pub enum ClientConfigError {
    /// Invalid URL provided in configuration.
    ///
    /// This error occurs when the provided URL string cannot be parsed
    /// according to URL standards.
    #[display("Invalid URL: {}", _0)]
    InvalidUrl(#[error(source)] url::ParseError),

    /// URL is missing required components or has unsupported features.
    ///
    /// This error occurs when the URL is syntactically valid but missing
    /// required components (like a host) or uses unsupported schemes.
    #[display("Invalid URL: {}", _0)]
    #[error(ignore)]
    MalformedUrl(String),
}

/// Configuration for the OpenSOVD client.
///
/// This struct contains all the configuration options needed to create
/// and configure an OpenSOVD client connection.
///
/// # Examples
///
/// ```rust
/// use opensovd_client::{ClientConfig, Client};
/// use std::time::Duration;
///
/// // Create a basic config
/// let config = ClientConfig::new("http://localhost:9000".to_string());
///
/// // Or use the builder pattern for more control
/// let config = ClientConfig::builder("http://localhost:9000".to_string())
///     .timeout(Duration::from_secs(10))
///     .user_agent("my-client/1.0")
///     .build()
///     .unwrap();
///
/// let client = Client::from_config(config);
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The base URL of the SOVD server
    pub url: String,
    /// Request timeout duration
    pub timeout: Duration,
    /// User agent string for HTTP requests
    pub user_agent: String,
    /// OpenSSL connector for HTTPS connections
    #[cfg(feature = "openssl")]
    pub ssl_connector: Option<openssl::ssl::SslConnector>,
}

impl ClientConfig {
    /// Creates a new `ClientConfig` with default settings.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL of the SOVD server
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::ClientConfig;
    ///
    /// let config = ClientConfig::new("http://localhost:9000".to_string());
    /// ```
    pub fn new(url: String) -> Self {
        Self {
            url,
            timeout: Duration::from_secs(30),
            user_agent: "opensovd-client/0.1.0".to_string(),
            #[cfg(feature = "openssl")]
            ssl_connector: None,
        }
    }

    /// Creates a new `ClientConfigBuilder` for fluent configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL of the SOVD server
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::builder("http://localhost:9000".to_string())
    ///     .timeout(Duration::from_secs(10))
    ///     .user_agent("my-client/1.0")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder(url: String) -> ClientConfigBuilder {
        ClientConfigBuilder::new(url)
    }
}

/// Builder for `ClientConfig` that allows fluent configuration.
///
/// This builder provides a convenient way to construct a `ClientConfig`
/// with custom settings using method chaining.
///
/// # Examples
///
/// ```rust
/// use opensovd_client::ClientConfig;
/// use std::time::Duration;
///
/// let config = ClientConfig::builder("http://localhost:9000".to_string())
///     .timeout(Duration::from_secs(5))
///     .user_agent("my-custom-client/2.0")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfigBuilder {
    url: String,
    timeout: Duration,
    user_agent: String,
    #[cfg(feature = "openssl")]
    ssl_connector: Option<openssl::ssl::SslConnector>,
}

impl ClientConfigBuilder {
    /// Creates a new `ClientConfigBuilder`.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL of the SOVD server
    pub fn new(url: String) -> Self {
        Self {
            url,
            timeout: Duration::from_secs(30),
            user_agent: "opensovd-client/0.1.0".to_string(),
            #[cfg(feature = "openssl")]
            ssl_connector: None,
        }
    }

    /// Sets the request timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for requests
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the user agent string.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The user agent string for HTTP requests
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Configure OpenSSL settings for HTTPS connections.
    ///
    /// This method accepts a pre-configured `SslConnector` which allows for
    /// fine-grained control over TLS/SSL settings including certificates, protocols,
    /// cipher suites, and server verification.
    ///
    /// # Arguments
    ///
    /// * `ssl_connector` - A configured SslConnector with certificates and TLS settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use opensovd_client::ClientConfig;
    /// use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode, SslFiletype};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Configure SSL with client certificate verification (mTLS)
    /// let mut ssl_builder = SslConnector::builder(SslMethod::tls_client())?;
    /// ssl_builder.set_certificate_chain_file("client.pem")?;
    /// ssl_builder.set_private_key_file("client.key", SslFiletype::PEM)?;
    /// ssl_builder.set_ca_file("ca.pem")?;
    /// ssl_builder.set_verify(SslVerifyMode::PEER);
    ///
    /// let ssl_connector = ssl_builder.build();
    ///
    /// let config = ClientConfig::builder("https://example.com".to_string())
    ///     .openssl(ssl_connector)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "openssl")]
    pub fn openssl(mut self, ssl_connector: openssl::ssl::SslConnector) -> Self {
        self.ssl_connector = Some(ssl_connector);
        self
    }

    /// Builds the `ClientConfig`.
    ///
    /// # Errors
    ///
    /// Returns a `ClientConfigError` if the configuration is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let result = ClientConfig::builder("http://example.com".to_string())
    ///     .timeout(Duration::from_secs(5))
    ///     .build();
    /// assert!(result.is_ok());
    /// ```
    pub fn build(self) -> Result<ClientConfig, ClientConfigError> {
        // Validate the URL before creating the config
        let parsed_url = Url::parse(&self.url).map_err(ClientConfigError::InvalidUrl)?;

        // Validate that the URL has a host
        if parsed_url.host_str().is_none() {
            return Err(ClientConfigError::MalformedUrl(
                "URL must have a valid host".to_string(),
            ));
        }

        // Validate scheme
        match parsed_url.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(ClientConfigError::MalformedUrl(format!("Unsupported scheme: {scheme}")));
            }
        }

        Ok(ClientConfig {
            url: self.url,
            timeout: self.timeout,
            user_agent: self.user_agent,
            #[cfg(feature = "openssl")]
            ssl_connector: self.ssl_connector,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_client_config_with_openssl() {
        #[cfg(feature = "openssl")]
        {
            use openssl::ssl::{SslConnector, SslMethod};

            let ssl_connector = SslConnector::builder(SslMethod::tls_client()).unwrap().build();

            let config = ClientConfig::builder("https://example.com".to_string())
                .timeout(Duration::from_secs(10))
                .user_agent("test-client/1.0")
                .openssl(ssl_connector)
                .build()
                .unwrap();

            assert_eq!(config.url, "https://example.com");
            assert_eq!(config.timeout, Duration::from_secs(10));
            assert_eq!(config.user_agent, "test-client/1.0");
            assert!(config.ssl_connector.is_some());
        }

        #[cfg(not(feature = "openssl"))]
        {
            // Test still passes when openssl feature is disabled
            let config = ClientConfig::builder("https://example.com".to_string())
                .timeout(Duration::from_secs(10))
                .user_agent("test-client/1.0")
                .build()
                .unwrap();

            assert_eq!(config.url, "https://example.com");
            assert_eq!(config.timeout, Duration::from_secs(10));
            assert_eq!(config.user_agent, "test-client/1.0");
        }
    }

    #[test]
    #[cfg(feature = "openssl")]
    fn test_openssl_configuration() {
        use openssl::ssl::{SslConnector, SslMethod};

        let ssl_connector = SslConnector::builder(SslMethod::tls_client()).unwrap().build();

        let config = ClientConfig::builder("https://example.com".to_string())
            .openssl(ssl_connector)
            .build()
            .unwrap();

        assert!(config.ssl_connector.is_some());
    }
}
