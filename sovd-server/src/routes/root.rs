// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpRequest, HttpResponse, web};
use sovd_diagnostic::Diagnostic;
use sovd_models::IncludeSchemaQuery;
use sovd_models::entity::{EntityCapabilitiesResponse, EntityId as ModelEntityId, EntityRelationships};

use crate::response::create_api_response;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_capabilities))
        .route("/", web::get().to(list_capabilities));
}

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

#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use sovd_diagnostic::Diagnostic;
    use sovd_models::entity::EntityCapabilitiesResponse;

    use super::*;

    fn app(
        base_uri: crate::routes::BaseUri,
        diagnostic: Diagnostic,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .app_data(web::Data::new(base_uri))
            .app_data(web::Data::new(diagnostic))
            .configure(configure)
    }

    #[actix_web::test]
    async fn test_list_capabilities_success() {
        let base_uri = crate::routes::BaseUri("http://localhost".to_string());
        let diagnostic = Diagnostic::builder().build();

        let app = test::init_service(app(base_uri, diagnostic)).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let caps_resp: EntityCapabilitiesResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(caps_resp.entity.id, "");
        assert_eq!(caps_resp.entity.name, "SOVD Server");
        assert!(caps_resp.relationships.components.is_some());
        assert!(caps_resp.relationships.components.unwrap().ends_with("/v1/components"));
    }

    #[actix_web::test]
    async fn test_list_capabilities_with_schema() {
        let base_uri = crate::routes::BaseUri("http://localhost".to_string());
        let diagnostic = Diagnostic::builder().build();

        let app = test::init_service(app(base_uri, diagnostic)).await;

        let req = test::TestRequest::get().uri("/?include-schema=true").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("schema").is_some());
        assert!(json.get("id").is_some());
        assert!(json.get("name").is_some());

        let schema = json.get("schema").unwrap();
        assert!(schema.is_object());
    }
}
