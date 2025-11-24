// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpResponse, web};
use sovd_diagnostic::Diagnostic;
use sovd_models::entity::DataWriteRequest;
use sovd_models::{
    ApiResponse, IncludeSchemaQuery,
    data::{DataGroupQuery, DataResourceQuery, DataResourceResponse},
};
use tracing::debug;

use crate::convert::parse_categories;
use crate::response::create_api_response;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/data", web::get().to(list_data_resources))
        .route("/data/{data_id}", web::get().to(get_data_value))
        .route("/data/{data_id}", web::put().to(set_data_value))
        .route("/data-categories", web::get().to(list_data_categories))
        .route("/data-groups", web::get().to(list_data_groups));
}

/// List data resources for a component
/// GET /components/{component-id}/data?categories[]=currentData&groups[]=engine&include-schema=true
pub(super) async fn list_data_resources(
    id: web::Path<String>,
    query: Result<web::Query<DataResourceQuery>, actix_web::Error>,
    diagnostic: web::Data<Diagnostic>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let component_id = id.as_str();
    debug!(component_id = %component_id, "List data resources");

    let data_service = diagnostic.get_service::<dyn sovd_diagnostic::DataService>(component_id)?;

    let query = query.unwrap_or_else(|_| web::Query(DataResourceQuery::default()));
    let include_schema = query.include_schema;
    let categories = parse_categories(query.categories.as_deref());
    let groups = query.groups.clone().unwrap_or_default();
    debug!(
        component_id = %component_id,
        categories = ?categories,
        groups = ?groups,
        "Call data_service.list"
    );
    let items = data_service
        .list(component_id, categories, groups)
        .await
        .map_err(crate::response::ApiError::from)?;
    debug!(component_id = %component_id, count = items.len(), "Data service list returned items");
    let response = DataResourceResponse { items };
    debug!(component_id = %component_id, count = response.items.len(), "Return DataResourceResponse");

    Ok(create_api_response(response, include_schema))
}

/// Get a specific data value
/// GET /components/{component-id}/data/{data-id}?include-schema=true
pub(super) async fn get_data_value(
    path: web::Path<(String, String)>,
    query: Result<web::Query<DataResourceQuery>, actix_web::Error>,
    diagnostic: web::Data<Diagnostic>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let (component_id, data_id) = path.into_inner();

    let data_service = diagnostic.get_service::<dyn sovd_diagnostic::DataService>(&component_id)?;

    let query = query.unwrap_or_else(|_| web::Query(DataResourceQuery::default()));

    if query.include_schema {
        let data_with_schema = data_service
            .read_with_schema(&component_id, &data_id)
            .await
            .map_err(crate::response::ApiError::from)?;
        let response = ApiResponse {
            data: data_with_schema.data,
            schema: data_with_schema.schema,
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        let value = data_service
            .read(&component_id, &data_id)
            .await
            .map_err(crate::response::ApiError::from)?;
        let response = ApiResponse {
            data: value,
            schema: None,
        };
        Ok(HttpResponse::Ok().json(response))
    }
}

/// Set a specific data value
/// PUT /components/{id}/data/{data_id}
pub(super) async fn set_data_value(
    path: web::Path<(String, String)>,
    request: web::Json<DataWriteRequest>,
    diagnostic: web::Data<Diagnostic>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let (component_id, data_id) = path.into_inner();
    let write_request = request.into_inner();

    let data_service = diagnostic.get_service::<dyn sovd_diagnostic::DataService>(&component_id)?;

    data_service
        .write(&component_id, &data_id, write_request.data)
        .await
        .map_err(crate::response::ApiError::from)?;

    Ok(HttpResponse::NoContent().finish())
}

/// List data categories for a component
/// GET /components/{component-id}/data-categories
pub(super) async fn list_data_categories(
    id: web::Path<String>,
    diagnostic: web::Data<Diagnostic>,
    query: Result<web::Query<IncludeSchemaQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let query = query.unwrap_or_else(|_| web::Query(IncludeSchemaQuery::default()));
    let include_schema = query.include_schema;

    let component_id = id.as_str();

    let data_service = diagnostic.get_service::<dyn sovd_diagnostic::DataService>(component_id)?;

    let items = data_service
        .list_categories(component_id)
        .await
        .map_err(crate::response::ApiError::from)?;

    let response = sovd_models::data::DataCategoryResponse { items };

    Ok(create_api_response(response, include_schema))
}

/// List data groups for a component
/// GET /components/{component-id}/data-groups?category=currentData
pub(super) async fn list_data_groups(
    id: web::Path<String>,
    query: Result<web::Query<DataGroupQuery>, actix_web::Error>,
    diagnostic: web::Data<Diagnostic>,
) -> Result<HttpResponse, crate::response::ApiError> {
    let component_id = id.as_str();

    let data_service = diagnostic.get_service::<dyn sovd_diagnostic::DataService>(component_id)?;

    let query = query.unwrap_or_else(|_| web::Query(DataGroupQuery::default()));
    let include_schema = query.include_schema;

    let category = query
        .category
        .as_deref()
        .and_then(crate::convert::parse_single_category);

    let items = data_service
        .list_groups(component_id, category)
        .await
        .map_err(crate::response::ApiError::from)?;

    let response = sovd_models::data::DataGroupResponse { items };

    Ok(create_api_response(response, include_schema))
}
