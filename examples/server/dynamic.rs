// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Dynamic entity addition example - adds new ECU entities every 4 seconds
//
// Run with: cargo run --example dynamic-server

use std::net::TcpListener;
use std::sync::Arc;

use examples::Ecu;
use sovd_diagnostic::{DiagnosticBuilder, EntityEvent};
use sovd_server::{Server, ServerConfig};
use tokio::time::{Duration, interval};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    examples::init_logging();

    let diagnostic = DiagnosticBuilder::new().build()?;
    let diagnostic_clone = diagnostic.clone();

    let mut rx = diagnostic.entities().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                EntityEvent::Added { entity_id, entity } => {
                    tracing::info!(entity_id = ?entity_id, entity_name = ?entity.name(), "Entity added");
                }
                EntityEvent::Removed { entity_id, entity } => {
                    tracing::info!(entity_id = ?entity_id, entity_name = ?entity.name(), "Entity removed");
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(4));
        let mut counter = 1;

        loop {
            ticker.tick().await;
            let ecu_id = format!("ecu{counter}");
            let ecu_name = format!("ECU {counter}");
            let ecu = Ecu::new(ecu_id, ecu_name);
            diagnostic_clone.entities().add_entity(Arc::new(ecu));
            counter += 1;
        }
    });

    // Start SOVD server on http://127.0.0.1:9000/sovd
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    tracing::info!("Starting SOVD server on http://127.0.0.1:9000/sovd");

    let config = ServerConfig::builder()
        .diagnostic(Arc::new(diagnostic))
        .endpoint(sovd_server::Listener::Tcp(listener), None, vec![], "/sovd".into())
        .build()?;

    Server::new(config).start().await?;
    Ok(())
}
