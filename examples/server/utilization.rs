// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// System utilization example - provides cpu and memory usage data
//
// Run with: cargo run --example utilization-server
//
// CPU usage: curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components/system/data/cpu_usage | jq
// Memory usage: curl --silent --show-error http://127.0.0.1:9000/sovd/v1/components/system/data/memory_usage | jq
//
// Sampler dashboard:
//   1. Start the server: cargo run --example utilization-server
//   2. In another terminal: sampler -c examples/sampler.yaml

use std::net::TcpListener;
use std::sync::Arc;

use async_trait::async_trait;
use examples::Component;
use serde_json::json;
use sovd_diagnostic::{
    DiagnosticBuilder,
    data::{DataCategory, DataCategoryInformation, DataError, DataService, ReadValue, ValueGroup, ValueMetaData},
};
use sovd_server::{Server, ServerConfig};
use sysinfo::System;
use tokio::sync::Mutex;

struct SystemData {
    sys: Mutex<System>,
}

impl SystemData {
    fn new() -> Self {
        Self {
            sys: Mutex::new(System::new()),
        }
    }
}

#[async_trait]
impl DataService for SystemData {
    async fn list(
        &self,
        _entity_id: &str,
        _categories: Vec<DataCategory>,
        _groups: Vec<String>,
    ) -> Result<Vec<ValueMetaData>, DataError> {
        Ok(vec![
            ValueMetaData {
                id: "cpu_usage".to_string(),
                name: "CPU Usage".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec![],
                tags: vec![],
            },
            ValueMetaData {
                id: "memory_usage".to_string(),
                name: "Memory Usage".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec![],
                tags: vec![],
            },
        ])
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

    async fn read(&self, _entity_id: &str, data_id: &str) -> Result<ReadValue, DataError> {
        let mut sys = self.sys.lock().await;

        match data_id {
            "cpu_usage" => {
                sys.refresh_cpu_usage();
                let cpu_usage = sys.global_cpu_usage();
                Ok(ReadValue {
                    id: "cpu_usage".to_string(),
                    data: json!(cpu_usage),
                    errors: vec![],
                })
            }
            "memory_usage" => {
                sys.refresh_memory();
                let total = sys.total_memory();
                let used = sys.used_memory();
                let usage = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                Ok(ReadValue {
                    id: "memory_usage".to_string(),
                    data: json!(usage),
                    errors: vec![],
                })
            }
            _ => Err(DataError::not_found(data_id)),
        }
    }

    async fn write(&self, _entity_id: &str, data_id: &str, _value: serde_json::Value) -> Result<(), DataError> {
        Err(DataError::not_found(data_id))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    examples::init_logging();

    let diagnostic = DiagnosticBuilder::new()
        .with_entity(Component::new("system".to_string(), "System".to_string()), |ctx| {
            ctx.with_service(Arc::new(SystemData::new()) as Arc<dyn DataService>)
        })
        .build();

    let listener = TcpListener::bind("127.0.0.1:9000")?;
    tracing::info!("Starting SOVD server on http://127.0.0.1:9000/sovd");

    let config = ServerConfig::builder()
        .diagnostic(diagnostic)
        .endpoint(sovd_server::Listener::Tcp(listener), None, vec![], "/sovd".into())
        .build()?;

    Server::new(config).start().await?;
    Ok(())
}
