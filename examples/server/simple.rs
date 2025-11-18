// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Simple SOVD server example with a mocked engine component
//
// Run with: cargo run --example simple-server

use std::net::TcpListener;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use sovd_diagnostic::{
    DiagnosticBuilder, Entity,
    data::{DataCategory, DataCategoryInformation, DataError, DataService, DataValue, ValueGroup, ValueMetaData},
};
use sovd_server::{Server, ServerConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup basic logging
    tracing_subscriber::fmt::init();

    // Create diagnostic with engine component and data service
    let diagnostic = DiagnosticBuilder::new()
        .with_entity(Arc::new(Engine), |ctx| {
            ctx.with_service(Arc::new(EngineData) as Arc<dyn DataService>)
        })
        .build()?;

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

// Simple engine component
struct Engine;

impl Entity for Engine {
    fn id(&self) -> &str {
        "engine"
    }

    fn name(&self) -> &str {
        "Engine Control Unit"
    }

    fn tags(&self) -> &[String] {
        &[]
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }
}

// Simple read-only data service
struct EngineData;

#[async_trait]
impl DataService for EngineData {
    async fn list(
        &self,
        _entity_id: &str,
        _categories: Vec<DataCategory>,
        _groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError> {
        Ok(vec![ValueMetaData {
            id: "rpm".to_string(),
            name: "Engine RPM".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec![],
            tags: vec![],
        }])
    }

    async fn list_categories(&self, _entity_id: &str) -> Result<Vec<DataCategoryInformation>, DataError> {
        Ok(vec![DataCategoryInformation {
            item: DataCategory::CurrentData,
            category_translation_id: None,
        }])
    }

    async fn list_groups(
        &self,
        _entity_id: &str,
        _category: Option<DataCategory>,
    ) -> Result<Vec<ValueGroup>, DataError> {
        Ok(vec![])
    }

    async fn read(&self, _entity_id: &str, data_id: &str) -> Result<DataValue, DataError> {
        if data_id == "rpm" {
            Ok(DataValue {
                id: "rpm".to_string(),
                value: json!(850),
                errors: vec![],
            })
        } else {
            Err(DataError::not_found(data_id))
        }
    }

    async fn write(&self, _entity_id: &str, data_id: &str, _value: serde_json::Value) -> Result<(), DataError> {
        // Read-only - reject all writes
        Err(DataError::not_found(data_id))
    }
}
