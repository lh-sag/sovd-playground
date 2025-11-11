// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

//! SOVD Server implementation
//!
//! This crate provides HTTP server implementations for the SOVD project.
//! Currently supports Actix-web based server with REST API endpoints.

mod convert;
mod error;
mod middleware;
mod response;
mod routes;
mod server;
mod server_config;

// Re-export main types for convenience
pub use error::{Error, Result};
pub use server::Server;
pub use server_config::{AuthInfo, Listener, ServerConfig, ServerConfigBuilder};
