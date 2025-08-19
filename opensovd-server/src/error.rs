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

//! Error types and definitions for the OpenSOVD server
//!
//! This module contains the core error types used throughout the server,
//! including vendor-specific error codes and the ApiError wrapper.
//!
//! ## Overview
//!
//! This module provides framework-agnostic error types that can be used
//! across different parts of the application. The actix-web specific
//! implementations (like `ResponseError` and `Responder` traits) are
//! kept in the separate `response` module to maintain clean separation
//! of concerns.
//!
//! ## Main Types
//!
//! - `VendorError`: Enum defining vendor-specific error codes
//! - `ApiError`: Wrapper around `GenericError<VendorError>` with helper methods
//! - `Error`: General server error type for internal operations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::error::ApiError;
//! use opensovd_models::error::ErrorCode;
//!
//! // Create a standard SOVD error
//! let error = ApiError::new(ErrorCode::IncompleteRequest, "Missing required field");
//!
//! // Create a vendor-specific error
//! let error = ApiError::not_found("Resource not found");
//! ```

use derive_more::{Display, Error as DeriveError, From};
use opensovd_models::error::{ErrorCode, GenericError};
use serde::{Deserialize, Serialize};

/// Vendor-specific error codes for the OpenSOVD server.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub enum VendorError {
    /// Internal server error
    #[display("Internal server error")]
    InternalError,
    /// Resource not found
    #[display("Resource not found")]
    NotFound,
}

/// Wrapper type for GenericError to implement ResponseError
#[derive(Debug, Display, From)]
#[display("{:?}: {}", self.0.error_code, self.0.message)]
pub struct ApiError(pub GenericError<VendorError>);

impl std::error::Error for ApiError {}

impl ApiError {
    /// Creates a new ApiError with the given error code and message
    pub fn new(error_code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError(GenericError {
            error_code,
            message: message.into(),
            vendor_code: None,
            transaction_id: None,
            parameters: None,
        })
    }

    /// Creates a new ApiError with a vendor-specific error
    pub fn vendor(vendor_error: VendorError, message: impl Into<String>) -> Self {
        ApiError(GenericError {
            error_code: ErrorCode::VendorSpecific,
            message: message.into(),
            vendor_code: Some(vendor_error),
            transaction_id: None,
            parameters: None,
        })
    }

    /// Creates an internal server error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::vendor(VendorError::InternalError, message)
    }

    /// Creates a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::vendor(VendorError::NotFound, message)
    }

    /// Creates an insufficient access rights error
    pub fn forbidden(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::InsufficientAccessRights, message)
    }

    /// Creates an incomplete request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::IncompleteRequest, message)
    }

    /// Creates a server failure error
    pub fn server_failure(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::SovdServerFailure, message)
    }

    /// Creates a server misconfigured error
    pub fn server_misconfigured(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::SovdServerMisconfigured, message)
    }

    /// Creates a precondition not fulfilled error
    pub fn precondition_failed(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::PreconditionNotFulfilled, message)
    }

    /// Creates a service unavailable error
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::NotResponding, message)
    }

    /// Creates an authentication error with invalid signature code
    pub fn unauthorized(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::InvalidSignature, message)
    }
}

/// Error types for the OpenSOVD server.
#[derive(Debug, Display, From, DeriveError)]
pub enum Error {
    /// IO error occurred during server operations.
    #[display("IO error: {}", _0)]
    Io(std::io::Error),

    /// Invalid URI provided.
    #[display("Invalid URI: {}", _0)]
    InvalidUri(http::uri::InvalidUri),

    /// Invalid URI provided.
    #[display("Bad configuration: {}", _0)]
    #[error(ignore)]
    BadConfiguration(String),
}

/// Result type alias for server operations.
pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::unnecessary_literal_unwrap)]
#[cfg(test)]
mod tests {
    use opensovd_models::error::ErrorCode;

    use super::*;

    #[test]
    fn test_error_code_serialization() {
        // Test that ErrorCode serializes to kebab-case strings
        let error = ApiError::new(ErrorCode::SovdServerFailure, "Test error");
        let json = serde_json::to_string(&error.0).unwrap();
        assert!(json.contains("\"error_code\":\"sovd-server-failure\""));

        let error = ApiError::new(ErrorCode::VendorSpecific, "Vendor error");
        let json = serde_json::to_string(&error.0).unwrap();
        assert!(json.contains("\"error_code\":\"vendor-specific\""));
    }

    #[test]
    fn test_vendor_error_creation() {
        let error = ApiError::vendor(VendorError::InternalError, "Internal error");
        assert_eq!(error.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(error.0.vendor_code, Some(VendorError::InternalError));
        assert_eq!(error.0.message, "Internal error");
    }

    #[test]
    fn test_helper_methods() {
        let error = ApiError::forbidden("Access denied");
        assert_eq!(error.0.error_code, ErrorCode::InsufficientAccessRights);
        assert_eq!(error.0.message, "Access denied");
        assert!(error.0.vendor_code.is_none());

        let error = ApiError::bad_request("Missing parameter");
        assert_eq!(error.0.error_code, ErrorCode::IncompleteRequest);
        assert_eq!(error.0.message, "Missing parameter");
        assert!(error.0.vendor_code.is_none());

        let error = ApiError::unauthorized("Invalid token");
        assert_eq!(error.0.error_code, ErrorCode::InvalidSignature);
        assert_eq!(error.0.message, "Invalid token");
        assert!(error.0.vendor_code.is_none());

        let error = ApiError::not_found("Resource not found");
        assert_eq!(error.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(error.0.message, "Resource not found");
        assert_eq!(error.0.vendor_code, Some(VendorError::NotFound));
    }

    #[test]
    fn test_error_conversions() {
        // Test From<std::io::Error>
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let server_err: Error = io_err.into();
        assert!(matches!(server_err, Error::Io(_)));
        assert_eq!(server_err.to_string(), "IO error: Permission denied");

        // Test From<http::uri::InvalidUri>
        let uri_error = "not a valid uri".parse::<http::Uri>().unwrap_err();
        let server_err: Error = uri_error.into();
        assert!(matches!(server_err, Error::InvalidUri(_)));
    }

    #[test]
    fn test_question_mark_operator_with_io_error() {
        fn simulate_io_operation() -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
        }

        fn test_function() -> Result<()> {
            simulate_io_operation()?;
            Ok(())
        }

        let result = test_function();
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(e.to_string(), "File not found");
            }
            _ => panic!("Expected IO error"),
        }
    }
}
