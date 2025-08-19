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

//! Actix-web specific response handling for OpenSOVD server
//!
//! This module contains the actix-web specific implementations including
//! the Responder trait and ResponseError trait implementations.
//!
//! ## Overview
//!
//! This module provides the web framework-specific response handling logic,
//! separating it from the core error definitions in the `error` module.
//! This separation allows the error types to remain framework-agnostic
//! while this module handles the actix-web specific conversions.
//!
//! ## Main Types
//!
//! - `ApiResult<T>`: A wrapper type that implements actix-web's `Responder` trait
//! - `ResponseError` implementation for `ApiError`: Converts errors to HTTP responses
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::response::ApiResult;
//!
//! async fn handler() -> ApiResult<MyResponse> {
//!     // Return success
//!     ApiResult::ok(MyResponse { /* ... */ })
//!
//!     // Or return error
//!     ApiResult::err(ApiError::not_found("Resource not found"))
//! }
//! ```

use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, http::StatusCode};
use opensovd_models::{JsonSchema, error::ErrorCode};

use crate::error::{ApiError, VendorError};

/// Wrapper type for API results that implements Responder
pub struct ApiResult<T>(pub Result<T, ApiError>);

impl<T> ApiResult<T> {
    /// Creates a successful ApiResult
    pub fn ok(value: T) -> Self {
        ApiResult(Ok(value))
    }

    /// Creates an error ApiResult
    pub fn err(error: ApiError) -> Self {
        ApiResult(Err(error))
    }
}

impl<T> From<Result<T, ApiError>> for ApiResult<T> {
    fn from(result: Result<T, ApiError>) -> Self {
        ApiResult(result)
    }
}

/// Implementation of Responder for ApiResult to allow direct return from handlers
impl<T> Responder for ApiResult<T>
where
    T: serde::Serialize + JsonSchema + Clone + Send + Sync + 'static,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            Ok(data) => {
                // Check for include_schema query parameter
                let include_schema = req.query_string().contains("include-schema=true");
                let response = if include_schema {
                    let schema = T::schema().ok();
                    opensovd_models::ApiResponse { data, schema }
                } else {
                    opensovd_models::ApiResponse { data, schema: None }
                };
                HttpResponse::Ok().json(response)
            }
            Err(error) => {
                // error is already ApiError which implements ResponseError
                error.error_response()
            }
        }
    }
}

/// Implementation of ResponseError for ApiError
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(&self.0)
    }

    fn status_code(&self) -> StatusCode {
        // Map error codes to HTTP status codes
        match self.0.error_code {
            ErrorCode::IncompleteRequest => StatusCode::BAD_REQUEST,
            ErrorCode::InsufficientAccessRights => StatusCode::FORBIDDEN,
            ErrorCode::InvalidResponseContent => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::InvalidSignature => StatusCode::UNAUTHORIZED,
            ErrorCode::LockBroken => StatusCode::CONFLICT,
            ErrorCode::NotResponding => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::PreconditionNotFulfilled => StatusCode::PRECONDITION_FAILED,
            ErrorCode::SovdServerFailure => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::SovdServerMisconfigured => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::UpdateAutomatedNotSupported => StatusCode::NOT_IMPLEMENTED,
            ErrorCode::UpdateExecutionInProgress => StatusCode::CONFLICT,
            ErrorCode::UpdatePreparationInProgress => StatusCode::CONFLICT,
            ErrorCode::UpdateProcessInProgress => StatusCode::CONFLICT,
            ErrorCode::VendorSpecific => {
                // Check vendor code for more specific status
                if let Some(ref vendor_code) = self.0.vendor_code {
                    match vendor_code {
                        VendorError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
                        VendorError::NotFound => StatusCode::NOT_FOUND,
                    }
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            ErrorCode::ErrorResponse => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use opensovd_models::error::ErrorCode;

    use super::*;
    use crate::error::ApiError;

    #[test]
    fn test_api_result_helpers() {
        let result: ApiResult<String> = ApiResult::ok("test".to_string());
        assert!(result.0.is_ok());

        let result: ApiResult<String> = ApiResult::err(ApiError::internal_error("Internal error"));
        assert!(result.0.is_err());
    }

    #[test]
    fn test_status_code_mapping() {
        let error = ApiError::new(ErrorCode::IncompleteRequest, "Bad request");
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);

        let error = ApiError::new(ErrorCode::InsufficientAccessRights, "Forbidden");
        assert_eq!(error.status_code(), StatusCode::FORBIDDEN);

        let error = ApiError::vendor(VendorError::InternalError, "Internal error");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_status() {
        let error = ApiError::new(ErrorCode::NotResponding, "Service unavailable");
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let error = ApiError::vendor(VendorError::InternalError, "Internal error");
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
