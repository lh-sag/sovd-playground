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

use clap::Parser;
use opensovd_server::{Server, ServerConfig};
use opensovd_tracing::info;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod, SslVerifyMode};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use url::Url;

mod cli;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
struct OpenSovdInfo {
    version: String,
    name: String,
    features: Vec<String>,
}

fn get_enabled_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "tracing")]
    features.push("tracing".to_string());

    #[cfg(feature = "ui")]
    features.push("ui".to_string());

    #[cfg(feature = "openssl")]
    features.push("openssl".to_string());

    features
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    let url: Url = args.url.parse()?;
    let vendor_info = OpenSovdInfo {
        version: cli::VERSION.to_string(),
        name: "OpenSOVD Gateway".to_string(),
        features: get_enabled_features(),
    };
    // Extract path from URL for server configuration
    let uri_path = match url.path() {
        "" | "/" => "/".to_string(),
        path => path.to_string(),
    };

    let mut config_builder = ServerConfig::builder_with_vendor_type::<OpenSovdInfo>()
        .vendor_info(vendor_info)
        .uri_path(&uri_path);

    // Configure listener based on URL scheme
    match url.scheme() {
        "http" => {
            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(9000);
            let listener = TcpListener::bind(format!("{}:{}", host, port))?;
            config_builder = config_builder.listen(listener);
        }

        #[cfg(feature = "openssl")]
        "https" => {
            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(9001);
            let listener = TcpListener::bind(format!("{}:{}", host, port))?;
            config_builder = config_builder.listen_openssl(listener, ssl_builder(&args.ssl)?);
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
            let socket_path =
                urlencoding::decode(encoded_socket_path).map_err(|e| format!("Failed to decode socket path: {}", e))?;
            let listener = UnixListener::bind(socket_path.as_ref())?;
            config_builder = config_builder.listen_uds(listener);
        }

        scheme => {
            return Err(format!("Unsupported URL scheme: {}", scheme).into());
        }
    }

    let server = Server::<OpenSovdInfo>::new(config_builder.build());
    info!(version = cli::VERSION, url = args.url, "Starting OpenSOVD server");

    server.start().await?;
    Ok(())
}

#[cfg(feature = "openssl")]
fn ssl_builder(ssl_args: &cli::SslArgs) -> std::result::Result<SslAcceptorBuilder, Box<dyn std::error::Error>> {
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
    let fail_if_no_peer_cert = !ssl_args.no_peer_cert;
    if fail_if_no_peer_cert {
        mode |= SslVerifyMode::FAIL_IF_NO_PEER_CERT;
    }
    builder.set_verify(mode);

    Ok(builder)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(serve())
}
