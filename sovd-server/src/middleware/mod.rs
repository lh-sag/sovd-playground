// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

//! HTTP middleware components for SOVD server.
//!
//! This module provides HTTP middleware layers for request processing, including:
//! - Request tracing and logging
//! - Authentication and authorization
//! - Request/response transformation
//! - Error handling and recovery
//!
//! The middleware is designed to be composable and follows Actix Web middleware patterns.

pub(crate) mod auth;
pub(crate) mod tracing;
