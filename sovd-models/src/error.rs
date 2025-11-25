// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use std::collections::HashMap;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
pub struct GenericError {
    pub error_code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
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
