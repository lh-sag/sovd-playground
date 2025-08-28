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

//! HTTP middleware components for OpenSOVD server.
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
