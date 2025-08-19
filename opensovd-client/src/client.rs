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

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::{Method, Request, Uri};
use hyper_util::rt::TokioIo;
use opensovd_models::version::VersionResponse;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslConnector, SslMethod};
use tokio::net::TcpStream;
#[cfg(feature = "openssl")]
use tokio_openssl::SslStream;
use url::Url;

use crate::client_config::ClientConfig;
use crate::error::Error;

/// OpenSOVD client for communicating with SOVD diagnostic servers.
///
/// The client provides methods to interact with SOVD servers, including
/// retrieving version information and performing diagnostic operations.
///
/// # Examples
///
/// ```rust
/// use opensovd_client::{Client, ClientConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a client with default configuration from URL
/// let client = Client::new("http://localhost:9000")?;
///
/// // Or create with custom configuration
/// let config = ClientConfig::builder("http://localhost:9000".to_string())
///     .build()?;
/// let client = Client::from_config(config);
///
/// // Get version information
/// let version_info = client.version_info::<opensovd_models::version::VendorInfo>().await?;
/// if let Some(info) = version_info.sovd_info.first() {
///     println!("Server version: {}", info.version);
/// }
/// # Ok(())
/// # }
/// ```
pub struct Client {
    config: ClientConfig,
}

impl Client {
    /// Creates a new client with default configuration from a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL of the SOVD server
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the URL is invalid or configuration fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::Client;
    ///
    /// let client = Client::new("http://localhost:9000").unwrap();
    /// ```
    pub fn new<U: Into<String>>(url: U) -> Result<Self, Error> {
        let config = ClientConfig::builder(url.into()).build().map_err(Error::Config)?;
        Ok(Self { config })
    }

    /// Creates a new client from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The client configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::{Client, ClientConfig};
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::builder("http://localhost:9000".to_string())
    ///     .timeout(Duration::from_secs(10))
    ///     .build()
    ///     .unwrap();
    ///
    /// let client = Client::from_config(config);
    /// ```
    pub fn from_config(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Retrieves version information from the SOVD server.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type for vendor-specific information that implements `Deserialize`
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    /// - The connection to the server fails
    /// - The HTTP request fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opensovd_client::Client;
    /// use opensovd_models::version::VendorInfo;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:9000")?;
    /// let version_info = client.version_info::<VendorInfo>().await?;
    /// if let Some(info) = version_info.sovd_info.first() {
    ///     println!("Version: {}", info.version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn version_info<T>(&self) -> Result<VersionResponse<T>, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Parse base URL and construct version-info endpoint
        // URL is already validated in config, so this should not fail
        let base_url = Url::parse(&self.config.url).map_err(|e| Error::InvalidResponse(format!("Invalid URL: {e}")))?;

        let version_url = base_url
            .join("version-info")
            .map_err(|e| Error::InvalidResponse(format!("Failed to construct version-info URL: {e}")))?;

        // Extract host and port from the URL
        // Host is guaranteed to exist due to config validation
        let host = base_url
            .host_str()
            .ok_or_else(|| Error::InvalidResponse("Invalid URL: missing host".to_string()))?;
        let port = base_url
            .port()
            .unwrap_or(if base_url.scheme() == "https" { 443 } else { 80 });

        // Handle HTTPS vs HTTP
        if base_url.scheme() == "https" {
            #[cfg(feature = "openssl")]
            {
                return self.make_tls_request(host, port, version_url).await;
            }
            #[cfg(not(feature = "openssl"))]
            {
                return Err(Error::InvalidResponse(
                    "HTTPS is not supported. Enable the 'openssl' feature".to_string(),
                ));
            }
        }

        // HTTP connection
        let tcp_stream = tokio::time::timeout(self.config.timeout, TcpStream::connect(format!("{host}:{port}")))
            .await
            .map_err(|_| Error::InvalidResponse("Connection timeout".to_string()))?
            .map_err(|e| Error::InvalidResponse(format!("Connection failed: {e}")))?;

        let io = TokioIo::new(tcp_stream);

        // Perform HTTP handshake
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.map_err(Error::Http)?;

        // Spawn connection task
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection error: {err}");
            }
        });

        // Build request
        let uri = version_url.as_str().parse::<Uri>()?;
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("host", host)
            .header("user-agent", &self.config.user_agent)
            .header("content-type", "application/json")
            .body(Empty::<Bytes>::new())
            .map_err(Error::HttpError)?;

        // Send request
        let res = tokio::time::timeout(self.config.timeout, sender.send_request(req))
            .await
            .map_err(|_| Error::InvalidResponse("Request timeout".to_string()))?
            .map_err(Error::Http)?;

        // Check status
        if !res.status().is_success() {
            return Err(Error::InvalidResponse(format!(
                "HTTP {}: {}",
                res.status().as_u16(),
                res.status().canonical_reason().unwrap_or("Unknown error")
            )));
        }

        // Read response body
        let body = res.into_body().collect().await?.to_bytes();
        let version_response: VersionResponse<T> = serde_json::from_slice(&body)?;

        Ok(version_response)
    }

    /// Returns the base URL of the SOVD server.
    pub fn url(&self) -> &str {
        &self.config.url
    }

    #[cfg(feature = "openssl")]
    async fn make_tls_request<T>(
        &self,
        host: &str,
        port: u16,
        version_url: url::Url,
    ) -> Result<VersionResponse<T>, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Use SSL connector from configuration, or create a default one
        let ssl_connector = match &self.config.ssl_connector {
            Some(connector) => connector.clone(),
            None => {
                // Create a default SSL connector for HTTPS without custom config
                SslConnector::builder(SslMethod::tls_client())
                    .map_err(|e| Error::InvalidResponse(format!("Failed to create SSL connector: {e}")))?
                    .build()
            }
        };

        // Create TCP connection
        let tcp_stream = tokio::time::timeout(self.config.timeout, TcpStream::connect(format!("{host}:{port}")))
            .await
            .map_err(|_| Error::InvalidResponse("Connection timeout".to_string()))?
            .map_err(|e| Error::InvalidResponse(format!("Connection failed: {e}")))?;

        // Configure SSL connection
        let ssl_config = ssl_connector
            .configure()
            .map_err(|e| Error::InvalidResponse(format!("Failed to configure SSL: {e}")))?
            .into_ssl(host)
            .map_err(|e| Error::InvalidResponse(format!("Failed to create SSL context: {e}")))?;

        // Perform TLS handshake
        let mut ssl_stream = SslStream::new(ssl_config, tcp_stream)
            .map_err(|e| Error::InvalidResponse(format!("Failed to create SSL stream: {e}")))?;

        use std::pin::Pin;
        Pin::new(&mut ssl_stream)
            .connect()
            .await
            .map_err(|e| Error::InvalidResponse(format!("TLS handshake failed: {e}")))?;

        let io = TokioIo::new(ssl_stream);

        // Perform HTTP handshake
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.map_err(Error::Http)?;

        // Spawn connection task
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection error: {err}");
            }
        });

        // Build request
        let uri = version_url.as_str().parse::<Uri>()?;
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("host", host)
            .header("user-agent", &self.config.user_agent)
            .header("content-type", "application/json")
            .body(Empty::<Bytes>::new())
            .map_err(Error::HttpError)?;

        // Send request
        let res = tokio::time::timeout(self.config.timeout, sender.send_request(req))
            .await
            .map_err(|_| Error::InvalidResponse("Request timeout".to_string()))?
            .map_err(Error::Http)?;

        // Check status
        if !res.status().is_success() {
            return Err(Error::InvalidResponse(format!(
                "HTTP {}: {}",
                res.status().as_u16(),
                res.status().canonical_reason().unwrap_or("Unknown error")
            )));
        }

        // Read response body
        let body = res.into_body().collect().await?.to_bytes();
        let version_response: VersionResponse<T> = serde_json::from_slice(&body)?;

        Ok(version_response)
    }
}
