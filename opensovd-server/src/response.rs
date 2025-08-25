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

use actix_web::{HttpRequest, HttpResponse, ResponseError, http::StatusCode};
use derive_more::{Display, From};
use opensovd_models::{ApiResponse, JsonSchema, error::{ErrorCode, GenericError}};
use serde::Serialize;

/// Vendor-specific error codes for the OpenSOVD server.
#[derive(Debug, Display, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub(crate) enum VendorError {
    InternalError,
    NotFound,
}

#[derive(Debug, Display, From)]
#[display("{:?}: {}", self.0.error_code, self.0.message)]
pub(crate) struct ApiError(pub GenericError<VendorError>);

impl std::error::Error for ApiError {}

impl ApiError {
    pub fn new(error_code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError(GenericError {
            error_code,
            message: message.into(),
            vendor_code: None,
            transaction_id: None,
            parameters: None,
        })
    }

    pub fn vendor(vendor_error: VendorError, message: impl Into<String>) -> Self {
        ApiError(GenericError {
            error_code: ErrorCode::VendorSpecific,
            message: message.into(),
            vendor_code: Some(vendor_error),
            transaction_id: None,
            parameters: None,
        })
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::vendor(VendorError::InternalError, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::vendor(VendorError::NotFound, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::InsufficientAccessRights, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::IncompleteRequest, message)
    }

    pub fn server_failure(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::SovdServerFailure, message)
    }

    pub fn server_misconfigured(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::SovdServerMisconfigured, message)
    }

    pub fn precondition_failed(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::PreconditionNotFulfilled, message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::NotResponding, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        ApiError::new(ErrorCode::InvalidSignature, message)
    }
}

/// Helper function to create an API response with optional schema inclusion
///
/// This function checks the request query string for `include-schema=true` and
/// automatically wraps the response data in an `ApiResponse` with schema if requested.
pub(crate) fn create_api_response<T>(data: T, req: &HttpRequest) -> HttpResponse
where
    T: serde::Serialize + JsonSchema + Clone + Send + Sync + 'static,
{
    let include_schema = req.query_string().contains("include-schema=true");
    let response = if include_schema {
        let schema = T::schema().ok();
        ApiResponse { data, schema }
    } else {
        ApiResponse { data, schema: None }
    };
    HttpResponse::Ok().json(response)
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


    #[test]
    fn test_api_error_default() {
        let error = ApiError::new(ErrorCode::SovdServerFailure, "test");
        assert_eq!(error.0.error_code, ErrorCode::SovdServerFailure);
        assert_eq!(error.0.message, "test");
        assert!(error.0.vendor_code.is_none());
        assert!(error.0.transaction_id.is_none());
        assert!(error.0.parameters.is_none());
    }

    #[test]
    fn test_status_code_exhaustive() {
        assert_eq!(ApiError::new(ErrorCode::ErrorResponse, "msg").status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::new(ErrorCode::IncompleteRequest, "msg").status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::new(ErrorCode::InsufficientAccessRights, "msg").status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::new(ErrorCode::InvalidResponseContent, "msg").status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(ApiError::new(ErrorCode::InvalidSignature, "msg").status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::new(ErrorCode::LockBroken, "msg").status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::new(ErrorCode::NotResponding, "msg").status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(ApiError::new(ErrorCode::PreconditionNotFulfilled, "msg").status_code(), StatusCode::PRECONDITION_FAILED);
        assert_eq!(ApiError::new(ErrorCode::SovdServerFailure, "msg").status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::new(ErrorCode::SovdServerMisconfigured, "msg").status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::new(ErrorCode::UpdateAutomatedNotSupported, "msg").status_code(), StatusCode::NOT_IMPLEMENTED);
        assert_eq!(ApiError::new(ErrorCode::UpdateExecutionInProgress, "msg").status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::new(ErrorCode::UpdatePreparationInProgress, "msg").status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::new(ErrorCode::UpdateProcessInProgress, "msg").status_code(), StatusCode::CONFLICT);
        
        assert_eq!(ApiError::vendor(VendorError::InternalError, "msg").status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ApiError::vendor(VendorError::NotFound, "msg").status_code(), StatusCode::NOT_FOUND);
        
        let mut vendor_no_code = ApiError::new(ErrorCode::VendorSpecific, "msg");
        vendor_no_code.0.vendor_code = None;
        assert_eq!(vendor_no_code.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response() {
        let error = ApiError::new(ErrorCode::IncompleteRequest, "test message");
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let error = ApiError::vendor(VendorError::NotFound, "resource missing");
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_api_error_all_methods() {
        let vendor_err = ApiError::vendor(VendorError::InternalError, "vendor");
        assert_eq!(vendor_err.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(vendor_err.0.message, "vendor");
        assert_eq!(vendor_err.0.vendor_code, Some(VendorError::InternalError));
        assert!(vendor_err.0.transaction_id.is_none());
        assert!(vendor_err.0.parameters.is_none());

        let internal = ApiError::internal_error("internal");
        assert_eq!(internal.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(internal.0.message, "internal");
        assert_eq!(internal.0.vendor_code, Some(VendorError::InternalError));
        assert!(internal.0.transaction_id.is_none());
        assert!(internal.0.parameters.is_none());

        let not_found = ApiError::not_found("missing");
        assert_eq!(not_found.0.error_code, ErrorCode::VendorSpecific);
        assert_eq!(not_found.0.message, "missing");
        assert_eq!(not_found.0.vendor_code, Some(VendorError::NotFound));
        assert!(not_found.0.transaction_id.is_none());
        assert!(not_found.0.parameters.is_none());

        let forbidden = ApiError::forbidden("denied");
        assert_eq!(forbidden.0.error_code, ErrorCode::InsufficientAccessRights);
        assert_eq!(forbidden.0.message, "denied");
        assert!(forbidden.0.vendor_code.is_none());
        assert!(forbidden.0.transaction_id.is_none());
        assert!(forbidden.0.parameters.is_none());

        let bad_req = ApiError::bad_request("invalid");
        assert_eq!(bad_req.0.error_code, ErrorCode::IncompleteRequest);
        assert_eq!(bad_req.0.message, "invalid");
        assert!(bad_req.0.vendor_code.is_none());
        assert!(bad_req.0.transaction_id.is_none());
        assert!(bad_req.0.parameters.is_none());

        let failure = ApiError::server_failure("failed");
        assert_eq!(failure.0.error_code, ErrorCode::SovdServerFailure);
        assert_eq!(failure.0.message, "failed");
        assert!(failure.0.vendor_code.is_none());
        assert!(failure.0.transaction_id.is_none());
        assert!(failure.0.parameters.is_none());

        let misconfig = ApiError::server_misconfigured("config");
        assert_eq!(misconfig.0.error_code, ErrorCode::SovdServerMisconfigured);
        assert_eq!(misconfig.0.message, "config");
        assert!(misconfig.0.vendor_code.is_none());
        assert!(misconfig.0.transaction_id.is_none());
        assert!(misconfig.0.parameters.is_none());

        let precond = ApiError::precondition_failed("precond");
        assert_eq!(precond.0.error_code, ErrorCode::PreconditionNotFulfilled);
        assert_eq!(precond.0.message, "precond");
        assert!(precond.0.vendor_code.is_none());
        assert!(precond.0.transaction_id.is_none());
        assert!(precond.0.parameters.is_none());

        let unavail = ApiError::service_unavailable("unavail");
        assert_eq!(unavail.0.error_code, ErrorCode::NotResponding);
        assert_eq!(unavail.0.message, "unavail");
        assert!(unavail.0.vendor_code.is_none());
        assert!(unavail.0.transaction_id.is_none());
        assert!(unavail.0.parameters.is_none());

        let unauth = ApiError::unauthorized("unauth");
        assert_eq!(unauth.0.error_code, ErrorCode::InvalidSignature);
        assert_eq!(unauth.0.message, "unauth");
        assert!(unauth.0.vendor_code.is_none());
        assert!(unauth.0.transaction_id.is_none());
        assert!(unauth.0.parameters.is_none());
    }
}
