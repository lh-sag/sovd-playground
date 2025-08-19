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

use derive_more::{Display, Error, From};

use crate::client_config::ClientConfigError;

/// Errors that can occur during OpenSOVD client operations.
///
/// This enum covers all possible errors that can occur when using the
/// OpenSOVD client, from configuration issues to network and protocol errors.
///
/// # Examples
///
/// ```rust
/// use opensovd_client::{Client, Error};
///
/// # async fn example() {
/// let client = Client::new("http://localhost:9000").unwrap();
/// match client.version_info::<opensovd_models::version::VendorInfo>().await {
///     Ok(version) => println!("Version: {:?}", version),
///     Err(Error::Config(config_err)) => {
///         println!("Configuration error: {}", config_err);
///     }
///     Err(Error::Http(http_err)) => {
///         println!("HTTP error: {}", http_err);
///     }
///     Err(Error::InvalidResponse(msg)) => {
///         println!("Invalid response: {}", msg);
///     }
///     Err(e) => println!("Other error: {}", e),
/// }
/// # }
/// ```
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// HTTP client errors from the underlying hyper client.
    ///
    /// This includes connection errors, timeout errors, and other
    /// low-level HTTP client issues.
    #[display("HTTP error: {}", _0)]
    Http(hyper::Error),

    /// URI parsing errors when constructing HTTP requests.
    ///
    /// This occurs when the constructed request URI is invalid.
    #[display("URI error: {}", _0)]
    Uri(hyper::http::uri::InvalidUri),

    /// HTTP protocol errors when building requests.
    ///
    /// This includes errors in HTTP headers, methods, or other protocol elements.
    #[display("HTTP error: {}", _0)]
    HttpError(hyper::http::Error),

    /// JSON serialization/deserialization errors.
    ///
    /// This occurs when the server response cannot be parsed as valid JSON
    /// or when the JSON structure doesn't match the expected format.
    #[display("JSON error: {}", _0)]
    Json(serde_json::Error),

    /// URL parsing errors during request construction.
    ///
    /// This occurs when URLs cannot be parsed or joined properly.
    #[display("URL parse error: {}", _0)]
    UrlParse(url::ParseError),

    /// Configuration errors during client setup.
    ///
    /// This includes invalid URLs, unsupported schemes, or other
    /// configuration validation failures.
    #[display("Configuration error: {}", _0)]
    Config(ClientConfigError),

    /// Invalid or unexpected server responses.
    ///
    /// This includes HTTP error status codes, malformed responses,
    /// connection timeouts, and other response-related issues.
    #[display("Invalid response: {}", _0)]
    #[error(ignore)]
    InvalidResponse(String),
}
