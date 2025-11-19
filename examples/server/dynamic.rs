// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::net::TcpListener;
use std::sync::Arc;

use sovd_diagnostic::DiagnosticBuilder;
use sovd_server::{Server, ServerConfig};
use tokio::time::{Duration, interval};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    examples::init_logging();

    let diagnostic = DiagnosticBuilder::new().build()?;
    let entities = Arc::clone(&diagnostic.entities);

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(4));
        let mut counter = 1;

        loop {
            ticker.tick().await;
            let ecu_id = format!("ecu{counter}");
            let ecu = Arc::new(Ecu::new(ecu_id.clone()));
            entities.add_entity(ecu);
            tracing::info!("Added {ecu_id}");
            counter += 1;
        }
    });

    let listener = TcpListener::bind("127.0.0.1:9000")?;
    tracing::info!("Starting SOVD server on http://127.0.0.1:9000/sovd");

    let config = ServerConfig::builder()
        .diagnostic(Arc::new(diagnostic))
        .endpoint(sovd_server::Listener::Tcp(listener), None, vec![], "/sovd".into())
        .build()?;

    Server::new(config).start().await?;
    Ok(())
}

struct Ecu {
    id: String,
}

impl Ecu {
    fn new(id: String) -> Self {
        Self { id }
    }
}

impl sovd_diagnostic::Entity for Ecu {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "ECU"
    }

    fn tags(&self) -> &[String] {
        &[]
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }
}
