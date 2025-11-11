// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

//! UI serving module for SOVD server
use actix_web::web;

/// Configures the UI endpoint based on the enabled features
pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(actix_files::Files::new("/ui", "./assets").index_file("index.html"));
}
