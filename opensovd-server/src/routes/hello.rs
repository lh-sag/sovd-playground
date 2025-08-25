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
use actix_web::{HttpResponse, web};
use serde_json::json;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/hello").route(web::get().to(get_hello)));
}

async fn get_hello() -> Result<HttpResponse, crate::response::ApiError> {
    let response = json!("Hello World");
    Ok(HttpResponse::Ok().json(response))
}
