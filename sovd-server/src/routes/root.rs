// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpRequest, HttpResponse, web};
use sovd_diagnostic::Diagnostic;
use sovd_models::IncludeSchemaQuery;
use sovd_models::entity::{EntityCapabilitiesResponse, EntityId as ModelEntityId, EntityRelationships};

use crate::response::create_api_response;

// GET /v1/ - Root discovery endpoint per ISO 17978-3 Section 7.6.3
pub(crate) async fn list_capabilities(
    req: HttpRequest,
    base_uri: web::Data<super::BaseUri>,
    diagnostic: web::Data<Diagnostic>,
    query: Result<web::Query<IncludeSchemaQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError> {
    // Handle query parameter with default
    let query = query.unwrap_or_else(|_| web::Query(IncludeSchemaQuery::default()));
    let include_schema = query.include_schema;

    let base_with_host = base_uri.resolve_uri(&req);
    let base = base_with_host.trim_end_matches('/');

    // Get the SovdServer entity from the repository (empty ID for root)
    let sovd_server = diagnostic
        .entities()
        .get_entity("")
        .ok_or_else(|| crate::response::ApiError::internal_error("SovdServer entity not found"))?;

    // Build the response using the entity
    let response = EntityCapabilitiesResponse {
        entity: ModelEntityId {
            id: sovd_server.id().to_string(), // Empty for SOVDServer root per ISO
            name: sovd_server.name().to_string(),
            translation_id: sovd_server.translation_id().map(|s| s.to_string()),
        },
        variant: None,
        relationships: EntityRelationships {
            // Entity collections at root level
            components: Some(format!("{base}/v1/components")),
            ..Default::default()
        },
        resources: Default::default(), // No resources at root level per ISO 17978-3
    };

    Ok(create_api_response(response, include_schema))
}
