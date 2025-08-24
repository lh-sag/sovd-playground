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
use opensovd_diagnostic::{Component, Diagnostic, HashMapDataResource, Resource};
use opensovd_models::data::StringDataCategory;
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

    // Create diagnostic system with example components using new ISO-compliant data resources
    
    // Create engine data resource with proper categorization
    let mut engine_data = HashMapDataResource::new();
    engine_data.add_data_item_with_metadata(
        "temperature".to_string(),
        "Engine Temperature".to_string(),
        StringDataCategory::CurrentData,
        vec!["engine".to_string(), "temperature".to_string()],
        vec!["celsius".to_string()],
        serde_json::json!({"value": 90.5, "unit": "celsius"}),
        false, // writable
    );
    engine_data.add_data_item_with_metadata(
        "rpm".to_string(),
        "Engine RPM".to_string(),
        StringDataCategory::CurrentData,
        vec!["engine".to_string(), "performance".to_string()],
        vec!["rpm".to_string()],
        serde_json::json!({"value": 2500, "unit": "rpm"}),
        false, // writable
    );
    engine_data.add_data_item_with_metadata(
        "oil_pressure".to_string(),
        "Oil Pressure".to_string(),
        StringDataCategory::CurrentData,
        vec!["engine".to_string(), "fluids".to_string()],
        vec!["pressure".to_string()],
        serde_json::json!({"value": 45, "unit": "psi"}),
        false, // writable
    );
    engine_data.add_data_item_with_metadata(
        "serial_number".to_string(),
        "Engine Serial Number".to_string(),
        StringDataCategory::IdentData,
        vec!["engine".to_string(), "identification".to_string()],
        vec!["serial".to_string()],
        serde_json::json!("ENG-2024-001"),
        true, // read-only
    );
    
    // Add vendor-specific engine data
    engine_data.add_data_item_with_metadata(
        "turbo_boost".to_string(),
        "Turbocharger Boost Pressure".to_string(),
        StringDataCategory::new_string_vendor("x-caterpillar-engine").unwrap(),
        vec!["engine".to_string(), "turbo".to_string()],
        vec!["pressure", "boost"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"value": 18.5, "unit": "psi", "max": 25.0}),
        false, // writable
    );
    engine_data.add_data_item_with_metadata(
        "def_level".to_string(),
        "Diesel Exhaust Fluid Level".to_string(),
        StringDataCategory::new_string_vendor("x-caterpillar-aftertreatment").unwrap(),
        vec!["engine".to_string(), "aftertreatment".to_string()],
        vec!["def", "fluid"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"level": 75, "unit": "percent"}),
        false, // writable
    );

    let engine_component = Component::new_with_resources(
        "engine-controller".to_string(),
        "Engine Control Unit".to_string(),
        Resource::with_data_resource(engine_data),
    );

    // Create transmission data resource with proper categorization
    let mut transmission_data = HashMapDataResource::new();
    transmission_data.add_data_item_with_metadata(
        "gear".to_string(),
        "Current Gear".to_string(),
        StringDataCategory::CurrentData,
        vec!["transmission".to_string(), "gearing".to_string()],
        vec!["gear".to_string()],
        serde_json::json!({"current": 3, "max": 6}),
        false, // writable
    );
    transmission_data.add_data_item_with_metadata(
        "fluid_temp".to_string(),
        "Transmission Fluid Temperature".to_string(),
        StringDataCategory::CurrentData,
        vec!["transmission".to_string(), "fluids".to_string()],
        vec!["temperature".to_string()],
        serde_json::json!({"value": 75, "unit": "celsius"}),
        false, // writable
    );
    transmission_data.add_data_item_with_metadata(
        "shift_mode".to_string(),
        "Shift Mode".to_string(),
        StringDataCategory::CurrentData,
        vec!["transmission".to_string(), "control".to_string()],
        vec!["mode".to_string()],
        serde_json::json!("automatic"),
        false, // writable
    );
    transmission_data.add_data_item_with_metadata(
        "part_number".to_string(),
        "Transmission Part Number".to_string(),
        StringDataCategory::IdentData,
        vec!["transmission".to_string(), "identification".to_string()],
        vec!["part".to_string()],
        serde_json::json!("TRANS-X900-2024"),
        true, // read-only
    );
    
    // Add vendor-specific transmission data
    transmission_data.add_data_item_with_metadata(
        "torque_converter_lockup".to_string(),
        "Torque Converter Lockup Status".to_string(),
        StringDataCategory::new_string_vendor("x-allison-transmission").unwrap(),
        vec!["transmission".to_string(), "torque-converter".to_string()],
        vec!["lockup", "status"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"locked": true, "slip_rpm": 25}),
        false, // writable
    );

    let transmission_component = Component::new_with_resources(
        "transmission-controller".to_string(),
        "Transmission Control Module".to_string(),
        Resource::with_data_resource(transmission_data),
    );

    // Create hydraulics system with vendor-specific categories
    let mut hydraulics_data = HashMapDataResource::new();
    hydraulics_data.add_data_item_with_metadata(
        "main_pressure".to_string(),
        "Main Hydraulic System Pressure".to_string(),
        StringDataCategory::new_string_vendor("x-liebherr-hydraulics").unwrap(),
        vec!["hydraulics".to_string(), "pressure".to_string()],
        vec!["main", "system"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"value": 350, "unit": "bar", "max": 420}),
        false, // writable
    );
    hydraulics_data.add_data_item_with_metadata(
        "pilot_pressure".to_string(),
        "Pilot Control Pressure".to_string(),
        StringDataCategory::new_string_vendor("x-liebherr-hydraulics").unwrap(),
        vec!["hydraulics".to_string(), "pilot".to_string()],
        vec!["pilot", "control"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"value": 28, "unit": "bar", "target": 30}),
        false, // writable
    );
    hydraulics_data.add_data_item_with_metadata(
        "boom_position".to_string(),
        "Boom Cylinder Position".to_string(),
        StringDataCategory::new_string_vendor("x-liebherr-construction").unwrap(),
        vec!["hydraulics".to_string(), "boom".to_string()],
        vec!["position", "cylinder"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!({"extension": 65, "unit": "percent"}),
        false, // writable
    );
    hydraulics_data.add_data_item_with_metadata(
        "fluid_temperature".to_string(),
        "Hydraulic Fluid Temperature".to_string(),
        StringDataCategory::CurrentData,
        vec!["hydraulics".to_string(), "fluids".to_string()],
        vec!["temperature".to_string()],
        serde_json::json!({"value": 45, "unit": "celsius", "warning_threshold": 80}),
        false, // writable
    );
    hydraulics_data.add_data_item_with_metadata(
        "pump_model".to_string(),
        "Hydraulic Pump Model".to_string(),
        StringDataCategory::IdentData,
        vec!["hydraulics".to_string(), "pump".to_string()],
        vec!["model", "identification"].iter().map(|s| s.to_string()).collect(),
        serde_json::json!("LH-PUMP-350-V2"),
        true, // read-only
    );
    
    let hydraulics_component = Component::new_with_resources(
        "hydraulics-controller".to_string(),
        "Hydraulic Control System".to_string(),
        Resource::with_data_resource(hydraulics_data),
    );

    let diagnostic = Diagnostic::builder()
        .add_component(engine_component)
        .add_component(transmission_component)
        .add_component(hydraulics_component)
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
