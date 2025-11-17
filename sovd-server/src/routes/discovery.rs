// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use actix_web::{HttpRequest, HttpResponse, web};
use sovd_diagnostic::{Diagnostic, EntityType};
use sovd_models::{
    IncludeSchemaQuery,
    entity::{
        EntityCapabilitiesResponse, EntityId as ModelEntityId, EntityQuery, EntityReference, EntityRelationships,
        EntityResponse,
    },
};

use crate::response::create_api_response;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/components")
            .route("", web::get().to(list_entities))
            .service(
                web::scope("/{id}")
                    .route("", web::get().to(list_capabilities))
                    .route("/hosts", web::get().to(list_hosts))
                    .configure(super::data::configure),
            ),
    );
}

// GET GET /{entity-collection}
async fn list_entities(
    req: HttpRequest,
    base_uri: web::Data<super::BaseUri>,
    diagnostic: web::Data<Diagnostic>,
    query: Result<web::Query<EntityQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let query = query.unwrap_or_else(|_| web::Query(EntityQuery::default()));

    // Collect components from the diagnostic system
    let mut items: Vec<EntityReference> = diagnostic
        .entities()
        .list_components()
        .into_iter()
        .map(|component| {
            crate::convert::entity_to_reference(component.as_ref(), &base_uri.resolve_uri(&req), EntityType::Component)
        })
        .collect();

    // Apply tag filtering if tags are specified
    if let Some(ref tags) = query.tags
        && !tags.is_empty()
    {
        // Filter entities that have at least one of the requested tags (OR logic)
        items.retain(|entity_ref| entity_ref.tags.iter().any(|entity_tag| tags.contains(entity_tag)));
    }

    // Return empty array instead of error when no items match
    let response = EntityResponse { items };

    Ok(create_api_response(response, query.include_schema))
}

// GET /components/{component-id}
async fn list_capabilities(
    req: HttpRequest,
    base_uri: web::Data<super::BaseUri>,
    component: web::Path<(String,)>,
    diagnostic: web::Data<Diagnostic>,
    query: Result<web::Query<IncludeSchemaQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError> {
    // Handle query parameter with default
    let query = query.unwrap_or_else(|_| web::Query(IncludeSchemaQuery::default()));
    let include_schema = query.include_schema;

    // Try to get the component from the diagnostic system
    let component_id = &component.0;

    // Early return if component not found - using the ? operator pattern
    let comp = diagnostic
        .entities()
        .get_entity(component_id)
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;

    let base = format!(
        "{}/v1/components/{}",
        base_uri.resolve_uri(&req).trim_end_matches('/'),
        component_id
    );

    // Populate all resource collections
    let resources = sovd_models::entity::Resources {
        data: Some(format!("{base}/data")),
        ..Default::default()
    };

    let relationships = EntityRelationships {
        subcomponents: None,
        // Only populate if component hosts apps
        hosts: if false {
            // hosts field removed from Entity trait
            Some(format!("{base}/hosts"))
        } else {
            None
        },
        // TODO: Add depends-on when dependency tracking is implemented
        depends_on: None,
        // TODO: Add belongs-to when area relationships are tracked
        belongs_to: None,
        ..Default::default()
    };

    let response = EntityCapabilitiesResponse {
        entity: ModelEntityId {
            id: comp.id().to_string(),
            name: comp.name().to_string(),
            translation_id: comp.translation_id().map(|s| s.to_string()),
        },
        variant: None,
        relationships,
        resources,
    };

    Ok(create_api_response(response, include_schema))
}

// GET /components/{component-id}/hosts
async fn list_hosts(
    _base_uri: web::Data<super::BaseUri>,
    component_id: web::Path<String>,
    diagnostic: web::Data<Diagnostic>,
    query: Result<web::Query<EntityQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let query = query.unwrap_or_else(|_| web::Query(EntityQuery::default()));

    // Get the component from the diagnostic system
    let _component = diagnostic
        .entities()
        .get_entity(&component_id)
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;

    // Get hosted apps (hosts field still exists but no apps to retrieve)
    let mut items: Vec<EntityReference> = Vec::new();
    // TODO: Implement once apps are re-added or hosts field is repurposed

    // Apply tag filtering if tags are specified (even though items is empty for now)
    if let Some(ref tags) = query.tags
        && !tags.is_empty()
    {
        items.retain(|entity_ref| entity_ref.tags.iter().any(|entity_tag| tags.contains(entity_tag)));
    }

    Ok(create_api_response(EntityResponse { items }, query.include_schema))
}
