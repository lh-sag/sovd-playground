// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

pub(crate) mod data;
pub(crate) mod discovery;
pub(crate) mod root;
#[cfg(feature = "ui")]
pub(crate) mod ui;
pub(crate) mod version;

use actix_web::http;

#[derive(Debug, Clone)]
pub(crate) struct BaseUri(pub String);

impl BaseUri {
    pub fn resolve_uri(&self, req: &actix_web::HttpRequest) -> String {
        let conn_info = req.connection_info();
        let scheme = conn_info.scheme();
        let host = conn_info.host();

        let path = self
            .0
            .parse::<http::Uri>()
            .ok()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| "/".to_string());

        format!("{scheme}://{host}{path}")
    }
}
