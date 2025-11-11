// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::entities::{Entity, SovdServer};
use crate::repository::EntityRepository;

#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum BuilderError {
    #[display("Entity with id '{}' already exists", _0)]
    DuplicateEntity(#[error(ignore)] String),
}

pub struct DiagnosticBuilder {
    repository: EntityRepository,
}

impl DiagnosticBuilder {
    pub fn new() -> Self {
        let repository = EntityRepository::new();
        let sovd_server = Arc::new(SovdServer::new("SOVD Server".to_string()));
        repository.add_entity(sovd_server);

        Self { repository }
    }

    pub fn add_entity(self, entity: Arc<dyn Entity>) -> Self {
        self.repository.add_entity(entity);
        self
    }

    pub fn build(self) -> Result<Diagnostic, BuilderError> {
        Ok(Diagnostic::new(self.repository))
    }
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Diagnostic {
    pub entities: Arc<EntityRepository>,
}

impl Diagnostic {
    pub fn new(entities: EntityRepository) -> Self {
        Self {
            entities: Arc::new(entities),
        }
    }

    pub fn empty() -> Self {
        Self::new(EntityRepository::new())
    }

    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::new()
    }

    pub fn entities(&self) -> &EntityRepository {
        &self.entities
    }
}

impl Clone for Diagnostic {
    fn clone(&self) -> Self {
        Self {
            entities: Arc::clone(&self.entities),
        }
    }
}
