// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

//! Conversion utilities between diagnostic layer types and server API types
//!
//! This module provides the bridging layer between sovd-diagnostic and
//! sovd-models without creating direct dependencies between them.

use sovd_diagnostic::{DataCategory, DataError};

use crate::response::ApiError;

pub fn parse_categories(cats: Option<&[String]>) -> Vec<DataCategory> {
    cats.map(|c| c.iter().filter_map(|s| parse_single_category(s)).collect())
        .unwrap_or_default()
}

pub fn parse_single_category(s: &str) -> Option<DataCategory> {
    match s {
        "identData" => Some(DataCategory::IdentData),
        "currentData" => Some(DataCategory::CurrentData),
        "storedData" => Some(DataCategory::StoredData),
        "sysInfo" => Some(DataCategory::SysInfo),
        // Vendor categories must start with "x-" but not "x-sovd-"
        s if s.starts_with("x-") && !s.starts_with("x-sovd-") => Some(DataCategory::Vendor(s.to_string())),
        _ => None,
    }
}

impl From<DataError> for ApiError {
    fn from(data_error: DataError) -> Self {
        let code = &data_error.error.error_code;
        let msg = &data_error.error.message;
        match code {
            sovd_models::error::ErrorCode::InsufficientAccessRights => ApiError::forbidden(msg.clone()),
            sovd_models::error::ErrorCode::IncompleteRequest => ApiError::bad_request(msg.clone()),
            sovd_models::error::ErrorCode::SovdServerFailure => ApiError::server_failure(msg.clone()),
            _ => ApiError::not_found(msg.clone()),
        }
    }
}
