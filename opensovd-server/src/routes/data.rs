// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

use actix_web::{HttpRequest, HttpResponse, web};
use opensovd_diagnostic::registry::ComponentRegistry;
use opensovd_models::entity::{DataResourceQuery, DataResourceResponse, DataValueResponse, DataWriteRequest};

use crate::convert::{data_error_to_response, data_items_to_resource_items};
use crate::response::create_api_response;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/components/{id}/data", web::get().to(list_data_resources))
        .route("/components/{id}/data/{data_id}", web::get().to(get_data_value))
        .route("/components/{id}/data/{data_id}", web::put().to(set_data_value))
        .route("/components/{id}/data-lists", web::get().to(list_data_resources));
}

/// List data resources for a component
/// GET /components/{id}/data?categories[]=currentData&groups[]=engine&include-schema=true
async fn list_data_resources(
    component_id: web::Path<String>,
    query: web::Query<DataResourceQuery>,
    registry: web::Data<ComponentRegistry>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError> {
    let component_id = component_id.as_str();
    let component = registry
        .get_component(component_id)
        .await
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;

    let query = query.into_inner();
    let categories = query.categories.unwrap_or_default();
    let groups = query.groups.unwrap_or_default();
    let data_items = component.list_data_items(&categories, &groups).await
        .map_err(|e| crate::response::ApiError::internal_error(format!("Failed to list data: {}", e)))?;
    let items = data_items_to_resource_items(data_items);
    let response = DataResourceResponse { items };

    Ok(create_api_response(response, &req))
}

/// Get a specific data value
/// GET /components/{id}/data/{data_id}?include-schema=true
async fn get_data_value(
    path: web::Path<(String, String)>,
    _query: Option<web::Query<DataResourceQuery>>,
    registry: web::Data<ComponentRegistry>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError> {
    let (component_id, data_id) = path.into_inner();
    let component = registry
        .get_component(&component_id)
        .await
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;
    
    let response = match component.read_data(&data_id).await {
        Ok(data) => DataValueResponse {
            id: data_id,
            data,
            errors: Vec::new(),
            schema: None,
        },
        Err(data_error) => {
            let error_response = data_error_to_response(data_error);

            DataValueResponse {
                id: data_id,
                data: serde_json::Value::Null,
                errors: vec![error_response],
                schema: None,
            }
        }
    };

    Ok(create_api_response(response, &req))
}

/// Set a specific data value
/// PUT /components/{id}/data/{data_id}
async fn set_data_value(
    path: web::Path<(String, String)>,
    request: web::Json<DataWriteRequest>,
    registry: web::Data<ComponentRegistry>,
    _req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError> {
    let (component_id, data_id) = path.into_inner();
    let write_request = request.into_inner();
    let component = registry
        .get_component(&component_id)
        .await
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;
    
    match component.write_data(&data_id, write_request.data).await {
        Ok(()) => {
            Ok(HttpResponse::NoContent().finish())
        }
        Err(data_error) => {
            Err(data_error.into())
        }
    }
}
