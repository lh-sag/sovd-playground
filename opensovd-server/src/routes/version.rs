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
use actix_web::{HttpRequest, HttpResponse, web};
use opensovd_models::JsonSchema;
use opensovd_models::version::VersionResponse;

use crate::response::create_api_response;

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
    req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + JsonSchema + Clone + Send + Sync + 'static,
{
    // Map ISO to API version.
    const VERSION: (&str, &str) = ("1.1", "v1");

    let response = VersionResponse {
        sovd_info: vec![
            opensovd_models::version::Info {
                version: VERSION.0.to_string(),
                base_uri: format!("{}/{}", base_uri.0.trim_end_matches('/'), VERSION.1),
                vendor_info: vendor_info.as_ref().clone(),
            },
            opensovd_models::version::Info {
                version: "1.2".to_string(),
                base_uri: format!("{}/{}", base_uri.0.trim_end_matches('/'), "v2"),
                vendor_info: vendor_info.as_ref().clone(),
            },
        ],
    };

    Ok(create_api_response(response, &req))
}
