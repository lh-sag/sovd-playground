// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::any::{Any, TypeId};
use std::sync::Arc;

use dashmap::DashMap;

use crate::entities::{Entity, SovdServer};
use crate::repository::EntityRepository;

#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum BuilderError {
    #[display("Entity with id '{}' already exists", _0)]
    DuplicateEntity(#[error(ignore)] String),
}

#[derive(Debug, Clone, PartialEq, derive_more::Display, derive_more::Error)]
pub enum ServiceError {
    #[display("Entity '{}' not found", entity_id)]
    EntityNotFound {
        #[error(ignore)]
        entity_id: String,
    },

    #[display("Service '{}' not found for entity '{}'", service_type, entity_id)]
    ServiceNotFound {
        #[error(ignore)]
        entity_id: String,
        #[error(ignore)]
        service_type: String,
    },
}

pub struct ServiceRegistry {
    services: DashMap<(String, TypeId), Box<dyn Any + Send + Sync>>,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
        }
    }

    pub fn register<T: Send + Sync + 'static + ?Sized>(&self, entity_id: &str, service: Arc<T>)
    where
        Arc<T>: Any + Send + Sync,
    {
        let type_id = TypeId::of::<Arc<T>>();
        self.services
            .insert((entity_id.to_string(), type_id), Box::new(service));
    }

    pub fn get<T: Send + Sync + 'static + ?Sized>(&self, entity_id: &str) -> Result<Arc<T>, ServiceError>
    where
        Arc<T>: Any + Send + Sync,
    {
        let type_id = TypeId::of::<Arc<T>>();
        self.services
            .get(&(entity_id.to_string(), type_id))
            .and_then(|s| s.downcast_ref::<Arc<T>>().map(Arc::clone))
            .ok_or_else(|| ServiceError::ServiceNotFound {
                entity_id: entity_id.to_string(),
                service_type: std::any::type_name::<T>().to_string(),
            })
    }
}

pub struct EntityContext {
    services: Vec<(TypeId, Box<dyn Any + Send + Sync>)>,
}

impl Default for EntityContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityContext {
    pub fn new() -> Self {
        Self { services: Vec::new() }
    }

    pub fn with_service<T: Send + Sync + 'static + ?Sized>(mut self, service: Arc<T>) -> Self
    where
        Arc<T>: Any + Send + Sync,
    {
        self.services.push((TypeId::of::<Arc<T>>(), Box::new(service)));
        self
    }
}

pub struct DiagnosticBuilder {
    repository: EntityRepository,
    services: ServiceRegistry,
}

impl DiagnosticBuilder {
    pub fn new() -> Self {
        let repository = EntityRepository::new();
        let sovd_server = Arc::new(SovdServer::new("SOVD Server".to_string()));
        repository.add_entity(sovd_server);

        Self {
            repository,
            services: ServiceRegistry::new(),
        }
    }

    pub fn add_entity(self, entity: impl Entity + 'static) -> Self {
        self.repository.add_entity(Arc::new(entity));
        self
    }

    pub fn with_entity<T, F>(self, entity: T, configure: F) -> Self
    where
        T: Entity + 'static,
        F: FnOnce(EntityContext) -> EntityContext,
    {
        let entity_id = entity.id().to_string();
        self.repository.add_entity(Arc::new(entity));

        let ctx = EntityContext::new();
        let ctx = configure(ctx);

        for (type_id, service) in ctx.services {
            self.services.services.insert((entity_id.clone(), type_id), service);
        }

        self
    }

    pub fn build(self) -> Result<Diagnostic, BuilderError> {
        Ok(Diagnostic::new(self.repository, self.services))
    }
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct DiagnosticInner {
    entities: EntityRepository,
    services: ServiceRegistry,
}

pub struct Diagnostic {
    inner: Arc<DiagnosticInner>,
}

impl Diagnostic {
    pub fn new(entities: EntityRepository, services: ServiceRegistry) -> Self {
        Self {
            inner: Arc::new(DiagnosticInner { entities, services }),
        }
    }

    pub fn empty() -> Self {
        Self::new(EntityRepository::new(), ServiceRegistry::new())
    }

    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::new()
    }

    pub fn entities(&self) -> &EntityRepository {
        &self.inner.entities
    }

    pub fn get_entity(&self, id: &str) -> Option<Arc<dyn Entity>> {
        self.inner.entities.get_entity(id)
    }

    pub fn list_entities(&self) -> Vec<Arc<dyn Entity>> {
        self.inner.entities.list_entities()
    }

    pub fn get_service<T: Send + Sync + 'static + ?Sized>(&self, entity_id: &str) -> Result<Arc<T>, ServiceError>
    where
        Arc<T>: Any + Send + Sync,
    {
        self.inner.services.get::<T>(entity_id)
    }

    pub fn has_service<T: Send + Sync + 'static + ?Sized>(&self, entity_id: &str) -> bool
    where
        Arc<T>: Any + Send + Sync,
    {
        self.inner.services.get::<T>(entity_id).is_ok()
    }
}

impl Clone for Diagnostic {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
