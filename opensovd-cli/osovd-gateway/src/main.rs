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
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::sync::Arc;

use clap::Parser;
#[cfg(not(feature = "config"))]
use opensovd_diagnostic::Diagnostic;
use opensovd_server::{AuthInfo, Server, ServerConfig};
#[cfg(feature = "openssl")]
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod, SslVerifyMode};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use url::Url;

mod cli;
#[cfg(feature = "config")]
mod config;
#[cfg(feature = "config-entities")]
mod hashmap_data_resource;

use libosovd::version::{ENABLED_FEATURES, VERSION};

const TARGET: &str = "gw";

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
        version: VERSION.to_string(),
        name: "OpenSOVD Gateway".to_string(),
        features: ENABLED_FEATURES.iter().map(|s| s.to_string()).collect(),
    };

    #[cfg(feature = "config")]
    let (diagnostic, config_auth) = {
        let cfg = match &args.config {
            Some(config_path) => {
                tracing::info!(target: TARGET,"Loading configuration from {:?}", config_path);
                config::Config::from_file(config_path)
                    .map_err(|e| format!("Failed to load configuration from {:?}: {}", config_path, e))?
            }
            None => {
                // No config specified, use builtin
                tracing::info!(target: TARGET,"Using builtin configuration");
                config::Config::builtin()
            }
        };

        let auth = cfg
            .auth
            .as_ref()
            .and_then(|a| a.jwt.as_ref())
            .map(|jwt| jwt.public_key_path.clone());

        (cfg.build_diagnostic()?, auth)
    };

    #[cfg(not(feature = "config"))]
    let (diagnostic, config_auth) = {
        tracing::info!(target: TARGET, "Using empty diagnostic (config feature disabled)");
        (Diagnostic::builder().build(), None)
    };

    let mut config_builder = ServerConfig::builder_with_vendor_type::<OpenSovdInfo>()
        .vendor_info(vendor_info)
        .diagnostic(Arc::new(diagnostic))
        .uri_path(urls.first().map(|u| u.path()).unwrap_or("/opensovd"));

    if let Some(ref jwt_key_path) = args.auth_jwt.or(config_auth) {
        let public_key_pem = std::fs::read(jwt_key_path)
            .map_err(|e| format!("Failed to read JWT public key from {}: {}", jwt_key_path, e))?;
        config_builder = config_builder.auth(AuthInfo { public_key_pem });
        tracing::info!(target: TARGET, jwt_key_path = %jwt_key_path, "JWT authentication enabled");
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
                let ssl = ssl_builder(&args.ssl)?;
                config_builder = config_builder.listen_openssl(listener, ssl);
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
    tracing::info!(target: TARGET, version = cli::VERSION, features = ?ENABLED_FEATURES, "Starting OpenSOVD server");
    tracing::info!(target: TARGET, urls = ?urls_str, "Serving requests on URLs");

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
    use tracing_subscriber::fmt;

    fmt().with_span_events(fmt::format::FmtSpan::CLOSE).compact().init();
    let tracer = tracing_log::LogTracer::new();
    log::set_boxed_logger(Box::new(tracer))?;
    log::set_max_level(log::LevelFilter::Debug);
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(serve())
}
