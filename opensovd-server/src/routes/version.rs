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
use opensovd_models::JsonSchema;
use opensovd_models::{IncludeSchemaParam, version::VersionResponse};

pub(crate) fn configure<T>(cfg: &mut web::ServiceConfig)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + JsonSchema + Clone + Send + Sync + 'static,
{
    cfg.service(web::resource("/version-info").route(web::get().to(get_version::<T>)));
}

/// Handles GET requests for `/version-info`.
///
/// This endpoint returns the current SOVD version information as a JSON object.
async fn get_version<T>(
    base_uri: web::Data<super::BaseUri>,
    vendor_info: web::Data<Option<T>>,
    include_schema: Result<web::Query<IncludeSchemaParam>, Error>,
) -> impl actix_web::Responder
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + JsonSchema + Clone + Send + Sync + 'static,
{
    // Map ISO to API version.
    const VERSION: (&str, &str) = ("1.1", "v1");
    let version_info = VersionResponse {
        sovd_info: vec![opensovd_models::version::Info {
            version: VERSION.0.to_string(),
            base_uri: base_uri.0.to_string() + VERSION.1,
            vendor_info: vendor_info.as_ref().clone(),
        }],
    };

    super::make_response(version_info, include_schema)
}
