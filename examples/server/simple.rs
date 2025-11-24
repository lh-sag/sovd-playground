// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Simple SOVD server example with a mocked engine component
//
// Run with: cargo run --example simple-server

use std::net::TcpListener;
use std::sync::Arc;

use examples::{Ecu, EngineData};
use sovd_diagnostic::{DiagnosticBuilder, data::DataService};
use sovd_server::{Server, ServerConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    examples::init_logging();

    let diagnostic = DiagnosticBuilder::new()
        .with_entity(
            Ecu::new("engine".to_string(), "Engine Control Unit".to_string()),
            |ctx| ctx.with_service(Arc::new(EngineData) as Arc<dyn DataService>),
        )
        .build();

    // Start SOVD server on http://127.0.0.1:9000/sovd
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    tracing::info!("Starting SOVD server on http://127.0.0.1:9000/sovd");

    let config = ServerConfig::builder()
        .diagnostic(diagnostic)
        .endpoint(sovd_server::Listener::Tcp(listener), None, vec![], "/sovd".into())
        .build()?;

    Server::new(config).start().await?;
    Ok(())
}
