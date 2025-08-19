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
use std::collections::HashMap;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::sync::Arc;

use clap::Parser;
use opensovd_diagnostic::{Component, Diagnostic, HashMapData, Resource};
use opensovd_server::{Server, ServerConfig};
use opensovd_tracing::info;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod, SslVerifyMode};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use url::Url;

mod cli;

const ENABLED_FEATURES: &[&str] = &[
    #[cfg(feature = "tracing")]
    "tracing",
    #[cfg(feature = "ui")]
    "ui",
    #[cfg(feature = "openssl")]
    "openssl",
];
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
struct OpenSovdInfo {
    version: String,
    name: String,
    features: Vec<String>,
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    let urls_str = args.get_urls();
    let mut urls = Vec::new();
    for url_str in &urls_str {
        let url: Url = url_str.parse()?;
        urls.push(url);
    }
    let vendor_info = OpenSovdInfo {
        version: cli::VERSION.to_string(),
        name: "OpenSOVD Gateway".to_string(),
        features: ENABLED_FEATURES.iter().map(|s| s.to_string()).collect(),
    };

    // Create diagnostic system with example components
    // Add a data resource to the engine component with some diagnostic values
    let engine_data = HashMapData::from_hashmap(HashMap::from([
        (
            "temperature".to_string(),
            serde_json::json!({"value": 90.5, "unit": "celsius"}),
        ),
        ("rpm".to_string(), serde_json::json!({"value": 2500, "unit": "rpm"})),
        (
            "oil_pressure".to_string(),
            serde_json::json!({"value": 45, "unit": "psi"}),
        ),
    ]));

    let engine_component = Component::new_with_resources(
        "engine-controller".to_string(),
        "Engine Control Unit".to_string(),
        Resource::with_data_resource(engine_data),
    );

    // Add a data resource to the transmission component
    let transmission_data = HashMapData::from_hashmap(HashMap::from([
        ("gear".to_string(), serde_json::json!({"current": 3, "max": 6})),
        (
            "fluid_temp".to_string(),
            serde_json::json!({"value": 75, "unit": "celsius"}),
        ),
        ("shift_mode".to_string(), serde_json::json!("automatic")),
    ]));

    let transmission_component = Component::new_with_resources(
        "transmission-controller".to_string(),
        "Transmission Control Module".to_string(),
        Resource::with_data_resource(transmission_data),
    );

    let diagnostic = Diagnostic::builder()
        .add_component(engine_component)
        .add_component(transmission_component)
        .build();

    let mut config_builder = ServerConfig::builder_with_vendor_type::<OpenSovdInfo>()
        .vendor_info(vendor_info)
        .diagnostic(Arc::new(diagnostic))
        .uri_path(urls.first().map(|u| u.path()).unwrap_or("/opensovd"));

    // Configure SSL settings if any HTTPS URLs are present
    #[cfg(feature = "openssl")]
    let has_https = urls.iter().any(|url| url.scheme() == "https");

    #[cfg(feature = "openssl")]
    if has_https {
        config_builder = config_builder.openssl(ssl_builder(&args.ssl)?);
    }

    // Configure listeners for each URL
    for url in &urls {
        match url.scheme() {
            "http" => {
                let host = url.host_str().unwrap_or("localhost");
                let port = url.port().unwrap_or(9000);
                let bind_addr = format!("{host}:{port}");
                let listener = TcpListener::bind(&bind_addr)?;
                config_builder = config_builder.listen(listener);
            }

            #[cfg(feature = "openssl")]
            "https" => {
                let host = url.host_str().unwrap_or("localhost");
                let port = url.port().unwrap_or(9001);
                let bind_addr = format!("{host}:{port}");
                let listener = TcpListener::bind(&bind_addr)?;
                config_builder = config_builder.listen_https(listener);
            }

            #[cfg(unix)]
            "unix" => {
                let socket_path = url.path();
                let listener = UnixListener::bind(socket_path)?;
                config_builder = config_builder.listen_uds(listener);
            }

            #[cfg(unix)]
            "http+unix" => {
                // For Docker-style URLs like http+unix://%2Fvar%2Frun%2Fdocker.sock/containers/json
                // the socket path is URL-encoded in the host part
                let encoded_socket_path = url
                    .host_str()
                    .ok_or("http+unix URL must have a host part containing the socket path")?;
                let socket_path = urlencoding::decode(encoded_socket_path)
                    .map_err(|e| format!("Failed to decode socket path: {e}"))?;
                let listener = UnixListener::bind(socket_path.as_ref())?;
                config_builder = config_builder.listen_uds(listener);
            }

            scheme => {
                return Err(format!("Unsupported URL scheme: {scheme}").into());
            }
        }
    }

    let config = config_builder.build()?;
    let server = Server::<OpenSovdInfo>::new(config);
    info!(version = cli::VERSION, "Starting OpenSOVD server");

    server.start().await?;
    Ok(())
}

#[cfg(feature = "openssl")]
fn ssl_builder(ssl_args: &libosovd::SslArgs) -> std::result::Result<SslAcceptorBuilder, Box<dyn std::error::Error>> {
    let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server())?;

    let key_path = ssl_args.key.as_ref().ok_or("Private key file is required for HTTPS")?;
    let cert_path = ssl_args.cert.as_ref().ok_or("Certificate file is required for HTTPS")?;

    builder.set_private_key_file(key_path, SslFiletype::PEM)?;
    builder.set_certificate_chain_file(cert_path)?;

    if let Some(ca) = &ssl_args.cacert {
        builder.set_ca_file(ca)?;
    }

    let mut mode = SslVerifyMode::NONE;
    if !ssl_args.insecure {
        mode |= SslVerifyMode::PEER;
    }
    if !ssl_args.no_peer_cert {
        mode |= SslVerifyMode::FAIL_IF_NO_PEER_CERT;
    }
    builder.set_verify(mode);

    Ok(builder)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tracing")]
    {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
        let tracer = tracing_log::LogTracer::new();
        log::set_boxed_logger(Box::new(tracer))?;
        log::set_max_level(log::LevelFilter::Debug);
    }

    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(serve())
}
