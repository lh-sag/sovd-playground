// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use examples::Ecu;
use sovd_diagnostic::{DiagnosticBuilder, EntityEvent};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    examples::init_logging();

    let diagnostic = DiagnosticBuilder::new().build()?;

    let mut rx = diagnostic.entities().subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                EntityEvent::Added { entity_id, entity } => {
                    tracing::info!("Entity added: {} ({})", entity.name(), entity_id);
                }
                EntityEvent::Removed { entity_id, entity } => {
                    tracing::info!("Entity removed: {} ({})", entity.name(), entity_id);
                }
            }
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    diagnostic
        .entities()
        .add_entity(Arc::new(Ecu::new("engine".to_string(), "Engine ECU".to_string())));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    diagnostic.entities().add_entity(Arc::new(Ecu::new(
        "transmission".to_string(),
        "Transmission ECU".to_string(),
    )));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    diagnostic.entities().remove_entity("engine");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
