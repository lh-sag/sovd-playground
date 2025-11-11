// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use derive_more::Display;

use crate::data::DataService;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum EntityType {
    Component,
    SovdServer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
#[display("{entity_type}:{id}")]
pub struct EntityId {
    pub entity_type: EntityType,
    pub id: String,
}

impl EntityId {
    pub fn new(entity_type: EntityType, id: String) -> Self {
        Self { entity_type, id }
    }

    pub fn component(id: String) -> Self {
        Self::new(EntityType::Component, id)
    }

    pub fn sovd_server() -> Self {
        Self::new(EntityType::SovdServer, String::new())
    }
}

/// Base trait for all SOVD entities
pub trait Entity: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn tags(&self) -> &[String];
    fn translation_id(&self) -> Option<&str> {
        None
    }

    /// Get the DataService for this entity, if available
    fn data_service(&self) -> Option<&dyn DataService> {
        None
    }
}

/// Special entity representing the SOVD server root
pub struct SovdServer {
    name: String,
}

impl SovdServer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Entity for SovdServer {
    fn id(&self) -> &str {
        // Empty ID for root per ISO 17978-3
        ""
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn tags(&self) -> &[String] {
        &[]
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }

    // No data services at root level per ISO 17978-3
    fn data_service(&self) -> Option<&dyn DataService> {
        None
    }
}
