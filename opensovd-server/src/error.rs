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

use thiserror::Error;

/// Error types for the OpenSOVD server.
#[derive(Error, Debug)]
pub enum Error {
    /// IO error occurred during server operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid URI provided.
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Invalid URI provided.
    #[error("Bad configuration: {0}")]
    BadConfiguration(String),
}

/// Result type alias for server operations.
pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::unnecessary_literal_unwrap)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let server_err: Error = io_err.into();
        assert!(matches!(server_err, Error::Io(_)));
        assert_eq!(server_err.to_string(), "IO error: Permission denied");
    }

    #[test]
    fn test_server_result() {
        let result: Result<String> = Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        )));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "IO error: File not found");
    }

    #[test]
    fn test_invalid_uri_error() {
        use http::Uri;

        // Test conversion from InvalidUri
        let invalid_uri_result: std::result::Result<Uri, _> = "not a valid uri".parse();
        let uri_error = invalid_uri_result.unwrap_err();
        let server_err: Error = uri_error.into();
        assert!(matches!(server_err, Error::InvalidUri(_)));
    }

    #[test]
    fn test_uri_error_conversion() {
        use http::Uri;

        let result: std::result::Result<Uri, http::uri::InvalidUri> = "http://[::1:80/".parse();
        match result {
            Err(e) => {
                let server_error: Error = e.into();
                assert!(matches!(server_error, Error::InvalidUri(_)));
            }
            Ok(_) => panic!("Expected parsing to fail"),
        }
    }

    #[test]
    fn test_automatic_io_error_conversion() {
        use std::io::ErrorKind;

        // Test that std::io::Error automatically converts to Error
        let io_error = std::io::Error::new(ErrorKind::PermissionDenied, "Access denied");
        let server_error: Error = io_error.into();

        assert!(matches!(server_error, Error::Io(_)));
        assert_eq!(server_error.to_string(), "IO error: Access denied");
    }

    #[test]
    fn test_question_mark_operator_with_io_error() {
        fn simulate_io_operation() -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
        }

        fn test_function() -> Result<()> {
            // Test that ? operator works automatically with io::Error -> Error conversion
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
