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

//! OpenSOVD Server implementation
//!
//! This crate provides HTTP server implementations for the OpenSOVD project.
//! Currently supports Actix-web based server with REST API endpoints.

pub mod convert;
pub mod error;
pub mod response;
pub mod routes;
mod server;
mod server_config;

// Re-export main types for convenience
pub use error::{ApiError, Error, Result, VendorError};
pub use response::ApiResult;
pub use server::Server;
pub use server_config::{ServerConfig, ServerConfigBuilder};
