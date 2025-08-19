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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct GenericError<T> {
    pub error_code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_code: Option<T>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    ErrorResponse,
    IncompleteRequest,
    InsufficientAccessRights,
    InvalidResponseContent,
    InvalidSignature,
    LockBroken,
    NotResponding,
    PreconditionNotFulfilled,
    SovdServerFailure,
    SovdServerMisconfigured,
    UpdateAutomatedNotSupported,
    UpdateExecutionInProgress,
    UpdatePreparationInProgress,
    UpdateProcessInProgress,
    VendorSpecific,
}
