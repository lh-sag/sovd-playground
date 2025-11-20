// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

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
        .build()?;

    let listener = TcpListener::bind("127.0.0.1:9000")?;
    tracing::info!("Starting SOVD server on http://127.0.0.1:9000/sovd");

    let shutdown = async move {
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => tracing::info!("Shutdown after 5 seconds"),
            _ = tokio::signal::ctrl_c() => tracing::info!("Shutdown from Ctrl+C"),
            _ = sigterm() => tracing::info!("Shutdown from SIGTERM"),
        }
    };

    let config = ServerConfig::builder()
        .diagnostic(Arc::new(diagnostic))
        .endpoint(sovd_server::Listener::Tcp(listener), None, vec![], "/sovd".into())
        .shutdown(shutdown)
        .build()?;

    Server::new(config).start().await?;
    Ok(())
}

#[cfg(unix)]
async fn sigterm() {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler")
        .recv()
        .await;
}

#[cfg(not(unix))]
async fn sigterm() {
    std::future::pending::<()>().await;
}
