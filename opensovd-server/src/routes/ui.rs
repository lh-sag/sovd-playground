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

//! UI serving module for OpenSOVD server
use actix_web::web;

/// Configures the UI endpoint based on the enabled features
pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(actix_files::Files::new("/ui", "./assets").index_file("index.html"));
}
