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
use clap::Parser;
use opensovd_server::{Server, ServerConfig};
use opensovd_tracing::info;
#[cfg(feature = "openssl")]
use openssl::error::ErrorStack;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod, SslVerifyMode};

use sovd::models::version::VendorInfo;
use std::env;
use std::net::TcpListener;
use tokio::runtime::Builder;

mod cli;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(serve())
}

#[cfg(feature = "openssl")]
fn ssl_builder(
    ssl_args: &cli::SslArgs,
) -> std::result::Result<SslAcceptorBuilder, ErrorStack> {
    let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server())?;

    builder.set_private_key_file(ssl_args.key.as_ref().unwrap(), SslFiletype::PEM)?;
    builder.set_certificate_chain_file(ssl_args.cert.as_ref().unwrap())?;
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

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let args = cli::Args::parse();
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    let vendor_info = VendorInfo {
        version: VERSION.to_string(),
        name: "OpenSOVD Gateway".to_string(),
    };

    let config = ServerConfig::builder().vendor_info(vendor_info).base_uri(&args.uri)?;
    #[cfg(not(feature = "openssl"))]
    let config = config.listen(listener);
    #[cfg(feature = "openssl")]
    let config = config.listen_openssl(
        listener,
        ssl_builder(&args.ssl)?,
    );
    let server = Server::new(config.build());
    info!(version = VERSION, uri = args.uri, "Starting OpenSOVD server");

    server.start().await?;
    Ok(())
}
