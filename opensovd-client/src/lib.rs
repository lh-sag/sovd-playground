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

//! OpenSOVD client library for communicating with SOVD diagnostic servers.
//!
//! This library provides a high-level HTTP client for interacting with OpenSOVD
//! (Service-Oriented Vehicle Diagnostics) servers. It handles configuration validation,
//! connection management, and protocol-specific operations.
//!
//! # Quick Start
//!
//! ```rust
//! use opensovd_client::{Client, ClientConfig};
//! use opensovd_models::version::VendorInfo;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with automatic configuration validation
//! let client = Client::new("http://localhost:9000")?;
//!
//! // Retrieve version information from the server
//! let version_info = client.version_info::<VendorInfo>().await?;
//! if let Some(info) = version_info.sovd_info.first() {
//!     println!("Server version: {}", info.version);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! For more advanced configuration, use the builder pattern:
//!
//! ```rust
//! use opensovd_client::{Client, ClientConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ClientConfig::builder("http://localhost:9000".to_string())
//!     .timeout(Duration::from_secs(10))
//!     .user_agent("my-diagnostic-tool/1.0")
//!     .build()?;
//!
//! let client = Client::from_config(config);
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! The library provides comprehensive error handling for configuration and runtime errors:
//!
//! ```rust
//! use opensovd_client::{Client, ClientConfig, Error, ClientConfigError};
//!
//! # async fn example() {
//! // Validation happens during client creation
//! match Client::new("invalid-url") {
//!     Ok(client) => {
//!         // Use client...
//!     }
//!     Err(Error::Config(ClientConfigError::InvalidUrl(e))) => {
//!         eprintln!("Invalid URL provided: {}", e);
//!     }
//!     Err(Error::Config(ClientConfigError::MalformedUrl(msg))) => {
//!         eprintln!("URL validation failed: {}", msg);
//!     }
//!     Err(e) => {
//!         eprintln!("Other error: {}", e);
//!     }
//! }
//!
//! // Or when using the builder pattern
//! match ClientConfig::builder("ftp://invalid-scheme.com".to_string()).build() {
//!     Ok(config) => {
//!         let client = Client::from_config(config);
//!         // Use client...
//!     }
//!     Err(ClientConfigError::MalformedUrl(msg)) => {
//!         eprintln!("Configuration error: {}", msg);
//!     }
//!     Err(e) => {
//!         eprintln!("Other config error: {}", e);
//!     }
//! }
//! # }
//! ```

pub mod client;
pub mod client_config;
pub mod error;

pub use client::Client;
pub use client_config::{ClientConfig, ClientConfigBuilder, ClientConfigError};
pub use error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("http://localhost:9000").unwrap();
        // Just verify we can create a client without panicking
        assert_eq!(client.url(), "http://localhost:9000");
    }

    #[test]
    fn test_client_config_builder() {
        use std::time::Duration;

        let config = ClientConfig::builder("http://localhost:9000".to_string())
            .timeout(Duration::from_secs(10))
            .user_agent("test-client/1.0")
            .build()
            .unwrap();

        let client = Client::from_config(config);
        assert_eq!(client.url(), "http://localhost:9000");
        assert_eq!(client.config().timeout, Duration::from_secs(10));
        assert_eq!(client.config().user_agent, "test-client/1.0");
    }

    #[test]
    fn test_client_config_defaults() {
        use std::time::Duration;

        let config = ClientConfig::new("http://example.com".to_string());
        assert_eq!(config.url, "http://example.com");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.user_agent, "opensovd-client/0.1.0");
    }

    #[test]
    fn test_invalid_url_validation() {
        // Test via Client::new
        let result = Client::new("not-a-valid-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_validation() {
        use std::time::Duration;

        // Test invalid URL
        let result = ClientConfig::builder("not-a-valid-url".to_string())
            .timeout(Duration::from_secs(10))
            .build();
        assert!(result.is_err());

        // Test unsupported scheme
        let result = ClientConfig::builder("ftp://example.com".to_string()).build();
        assert!(result.is_err());

        // Test missing host
        let result = ClientConfig::builder("http://".to_string()).build();
        assert!(result.is_err());

        // Test valid configuration
        let result = ClientConfig::builder("https://example.com".to_string())
            .timeout(Duration::from_secs(5))
            .user_agent("test-agent")
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_creation_methods() {
        use std::time::Duration;

        // Test Client::new with URL
        let client1 = Client::new("http://example.com").unwrap();
        assert_eq!(client1.url(), "http://example.com");

        // Test Client::from_config with builder pattern
        let config = ClientConfig::builder("http://example.com".to_string())
            .timeout(Duration::from_secs(15))
            .user_agent("test/1.0")
            .build()
            .unwrap();

        let client2 = Client::from_config(config);
        assert_eq!(client2.url(), "http://example.com");
        assert_eq!(client2.config().timeout, Duration::from_secs(15));
        assert_eq!(client2.config().user_agent, "test/1.0");

        // Test Client::from_config with ClientConfig::new
        let config = ClientConfig::new("http://example.com".to_string());
        let client3 = Client::from_config(config);
        assert_eq!(client3.url(), "http://example.com");
        assert_eq!(client3.config().timeout, Duration::from_secs(30)); // default
    }

    #[test]
    fn test_public_api_exports() {
        // Test that all types are properly exported and can be imported
        use std::time::Duration;

        // Test ClientConfigError import and usage
        let result = ClientConfig::builder("not-a-url".to_string()).build();
        assert!(matches!(result, Err(ClientConfigError::InvalidUrl(_))));

        let result = ClientConfig::builder("ftp://example.com".to_string()).build();
        assert!(matches!(result, Err(ClientConfigError::MalformedUrl(_))));

        // Test Error import and usage
        let result = Client::new("not-a-valid-url");
        assert!(matches!(result, Err(Error::Config(_))));

        // Test successful configuration creation and client usage
        let config = ClientConfig::builder("http://example.com".to_string())
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let _client = Client::from_config(config);
    }

    #[test]
    fn test_module_structure_and_reexports() {
        use std::time::Duration;

        // Test that Client is correctly re-exported from client_hyper
        let config = ClientConfig::builder("http://example.com".to_string())
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let client = Client::from_config(config);
        assert_eq!(client.url(), "http://example.com");

        // Test that configuration types are available from client module
        let config2 = ClientConfig::new("http://example.com".to_string());
        assert_eq!(config2.url, "http://example.com");

        // Test that errors are properly accessible
        let result = Client::new("invalid-url");
        assert!(matches!(result, Err(Error::Config(ClientConfigError::InvalidUrl(_)))));

        // Test that all public API types are accessible through re-exports
        let _: fn(ClientConfig) -> Client = Client::from_config;
    }

    #[test]
    fn test_client_new_into_string() {
        // Test that Client::new works with different string types via Into<String>

        // &str
        let client1 = Client::new("http://example.com").unwrap();
        assert_eq!(client1.url(), "http://example.com");

        // String
        let url = String::from("http://example.com");
        let client2 = Client::new(url).unwrap();
        assert_eq!(client2.url(), "http://example.com");

        // &String
        let url = String::from("http://example.com");
        let client3 = Client::new(&url).unwrap();
        assert_eq!(client3.url(), "http://example.com");

        // Test error handling with different string types
        let result1 = Client::new("invalid-url");
        assert!(result1.is_err());

        let invalid_url = String::from("ftp://invalid.com");
        let result2 = Client::new(invalid_url);
        assert!(result2.is_err());
    }
}
