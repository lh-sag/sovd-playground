// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpRequest, HttpResponse, web};
use awc::Client;
use tracing::{debug, error, info};

/// Hardcoded target URL for the proxy
const TARGET_BASE_URL: &str = "http://localhost:8080";

/// Headers that should not be forwarded (hop-by-hop headers)
const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(web::Data::new(Client::default()));
    cfg.service(
        web::scope("/proxy")
            .route("", web::route().to(proxy_handler))
            .route("/{path:.*}", web::route().to(proxy_handler))
    );
}

/// Build the Forwarded header according to RFC 7239
/// Format: Forwarded: for=192.0.2.43;host=example.com;proto=https
fn build_forwarded_header(req: &HttpRequest) -> String {
    let mut parts = Vec::new();
    
    // Add 'for' parameter (client IP)
    if let Some(peer_addr) = req.peer_addr() {
        // Format IPv6 addresses with brackets
        let for_value = match peer_addr.ip() {
            std::net::IpAddr::V6(v6) => format!("\"[{}]:{}\"", v6, peer_addr.port()),
            std::net::IpAddr::V4(v4) => format!("{}:{}", v4, peer_addr.port()),
        };
        parts.push(format!("for={}", for_value));
    }
    
    // Add 'host' parameter
    if let Some(host) = req.headers().get("host") {
        if let Ok(host_str) = host.to_str() {
            // Quote if contains special characters
            let host_value = if host_str.contains(';') || host_str.contains(',') {
                format!("\"{}\"", host_str)
            } else {
                host_str.to_string()
            };
            parts.push(format!("host={}", host_value));
        }
    }
    
    // Add 'proto' parameter
    let proto = if req.connection_info().scheme() == "https" { "https" } else { "http" };
    parts.push(format!("proto={}", proto));
    
    // Add 'by' parameter (proxy identifier) - optional
    // We could add the proxy server's address here if needed
    // parts.push(format!("by=\"{}\"", proxy_identifier));
    
    parts.join(";")
}

async fn proxy_handler(
    req: HttpRequest,
    payload: web::Payload,
    path: web::Path<String>,
    client: web::Data<Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let request_path = path.into_inner();
    let target_url = if request_path.is_empty() {
        TARGET_BASE_URL.to_string()
    } else {
        format!("{}/{}", TARGET_BASE_URL, request_path)
    };

    // Add query string if present
    let target_url = if let Some(query) = req.uri().query() {
        format!("{}?{}", target_url, query)
    } else {
        target_url
    };

    info!("Proxying request: {} {} -> {}", req.method(), req.path(), target_url);

    // Build the forwarded request
    let mut forwarded_req = client.request(req.method().clone(), &target_url);

    // Copy headers from original request, filtering out hop-by-hop headers
    for (header_name, header_value) in req.headers().iter() {
        let header_name_str = header_name.as_str().to_lowercase();
        if !HOP_BY_HOP_HEADERS.contains(&header_name_str.as_str()) {
            forwarded_req = forwarded_req.insert_header((header_name.clone(), header_value.clone()));
        }
    }

    // Add RFC 7239 Forwarded header
    let forwarded_value = build_forwarded_header(&req);
    
    // Append to existing Forwarded header if present
    if let Some(existing_forwarded) = req.headers().get("forwarded") {
        if let Ok(existing) = existing_forwarded.to_str() {
            forwarded_req = forwarded_req.insert_header((
                "forwarded",
                format!("{}, {}", existing, forwarded_value)
            ));
        } else {
            forwarded_req = forwarded_req.insert_header(("forwarded", forwarded_value));
        }
    } else {
        forwarded_req = forwarded_req.insert_header(("forwarded", forwarded_value));
    }

    // Send the request with streaming body
    let res = match forwarded_req.send_stream(payload).await {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to forward request: {}", err);
            return Ok(HttpResponse::BadGateway().body(format!("Proxy error: {}", err)));
        }
    };

    // Build the response to return to client
    let mut client_resp = HttpResponse::build(res.status());

    // Copy response headers, filtering out hop-by-hop headers
    for (header_name, header_value) in res.headers().iter() {
        let header_name_str = header_name.as_str().to_lowercase();
        if !HOP_BY_HOP_HEADERS.contains(&header_name_str.as_str()) {
            client_resp.insert_header((header_name.clone(), header_value.clone()));
        }
    }

    debug!("Proxy response status: {}", res.status());

    // Stream the response body back to client
    Ok(client_resp.streaming(res))
}