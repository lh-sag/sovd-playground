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
use actix_web::{Error, web};
use opensovd_models::{IncludeSchemaParam, JsonSchema};

pub(crate) mod entity;
#[cfg(feature = "ui")]
pub(crate) mod ui;
pub(crate) mod version;

#[derive(Debug, Clone)]
pub(crate) struct BaseUri(pub String);

/// Creates a response with optional schema inclusion.
///
/// This function takes data and an optional schema parameter, and returns
/// an HTTP response wrapped in JSON. If schema inclusion is requested and
/// successful, the schema will be included in the response.
pub fn make_response<T>(data: T, param: Result<web::Query<IncludeSchemaParam>, Error>) -> impl actix_web::Responder
where
    T: serde::Serialize + JsonSchema + Clone + Send + Sync + 'static,
{
    let include_schema = match param {
        Ok(p) => p.into_inner().include_schema,
        Err(_) => false,
    };
    if include_schema {
        let schema = T::schema().ok();
        web::Json(opensovd_models::ApiResponse { data, schema })
    } else {
        web::Json(opensovd_models::ApiResponse { data, schema: None })
    }
}
