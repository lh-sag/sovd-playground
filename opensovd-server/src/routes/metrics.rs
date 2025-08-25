// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpResponse, web};

#[cfg(feature = "metrics")]
pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(get_metrics));
}

#[cfg(not(feature = "metrics"))]
pub(crate) fn configure(_cfg: &mut web::ServiceConfig) {
    // No metrics endpoint when features are disabled
}

#[cfg(feature = "metrics")]
async fn get_metrics() -> HttpResponse {
    let metrics = opensovd_diagnostic::metrics::metrics_to_string();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

#[cfg(not(feature = "metrics"))]
async fn get_metrics() -> HttpResponse {
    HttpResponse::NotFound()
        .body("Metrics not enabled. Build with --features metrics")
}