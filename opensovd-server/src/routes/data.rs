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

use actix_web::web;
use opensovd_diagnostic::Diagnostic;
use opensovd_models::entity::{
    DataResourceResponse, DataValueResponse, DataWriteRequest,
    DataResourceQuery
};

use crate::convert::{
    parse_category_filters, data_items_to_resource_items, data_error_to_response
};
use crate::response::ApiResult;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/components/{id}/data", web::get().to(list_data_resources))
        .route("/components/{id}/data/{data_id}", web::get().to(get_data_value))
        .route("/components/{id}/data/{data_id}", web::put().to(set_data_value))
        .route("/components/{id}/data-lists", web::get().to(list_data_resources)); // Same as data endpoint for now
}


/// List data resources for a component
/// GET /components/{id}/data?categories[]=currentData&groups[]=engine&include-schema=true
async fn list_data_resources(
    component_id: web::Path<String>,
    query: web::Query<DataResourceQuery>,
    diagnostic: web::Data<Diagnostic>,
) -> ApiResult<DataResourceResponse> {
    let component_id = component_id.as_str();
    
    match diagnostic.get_component(component_id) {
        Some(component) => {
            match component.resource().get_data_resource() {
                Some(data_resource) => {
                    // Parse category filters using conversion utility
                    let category_filters = parse_category_filters(&query.categories);

                    // Get filtered data items
                    let data_items = data_resource.list_data_items(&category_filters, &query.groups);
                    
                    // Convert to response format using conversion utility
                    let items = data_items_to_resource_items(data_items);

                    ApiResult::ok(DataResourceResponse { items })
                }
                None => {
                    ApiResult::err(crate::error::ApiError::not_found(
                        format!("Component '{}' has no data resources", component_id)
                    ))
                }
            }
        }
        None => ApiResult::err(crate::error::ApiError::not_found(
            format!("Component '{}' not found", component_id)
        )),
    }
}

/// Get a specific data value
/// GET /components/{id}/data/{data_id}?include-schema=true
async fn get_data_value(
    path: web::Path<(String, String)>,
    query: web::Query<DataResourceQuery>,
    diagnostic: web::Data<Diagnostic>,
) -> ApiResult<DataValueResponse> {
    let (component_id, data_id) = path.into_inner();
    
    match diagnostic.get_component(&component_id) {
        Some(component) => {
            match component.resource().get_data_resource() {
                Some(data_resource) => {
                    match data_resource.read_data(&data_id) {
                        Ok(data) => {
                            let mut response = DataValueResponse {
                                id: data_id,
                                data,
                                errors: Vec::new(),
                                schema: None,
                            };

                            // Add schema if requested
                            if query.include_schema {
                                // TODO: Generate schema from data value
                                response.schema = Some(serde_json::json!({
                                    "type": "object",
                                    "description": "Auto-generated schema"
                                }));
                            }

                            ApiResult::ok(response)
                        }
                        Err(data_error) => {
                            // Convert DataError to API error response using conversion utility
                            let error_response = data_error_to_response(data_error);

                            let response = DataValueResponse {
                                id: data_id,
                                data: serde_json::Value::Null,
                                errors: vec![error_response],
                                schema: None,
                            };

                            ApiResult::ok(response)
                        }
                    }
                }
                None => ApiResult::err(crate::error::ApiError::not_found(
                    format!("Component '{}' has no data resources", component_id)
                )),
            }
        }
        None => ApiResult::err(crate::error::ApiError::not_found(
            format!("Component '{}' not found", component_id)
        )),
    }
}

/// Set a specific data value
/// PUT /components/{id}/data/{data_id}
async fn set_data_value(
    path: web::Path<(String, String)>,
    request: web::Json<DataWriteRequest>,
    diagnostic: web::Data<Diagnostic>,
) -> ApiResult<DataValueResponse> {
    let (component_id, data_id) = path.into_inner();
    
    // Note: We need mutable access to diagnostic, but actix-web Data is immutable
    // For now, we'll return an error. In a real implementation, we'd need to use
    // Arc<RwLock<Diagnostic>> or similar for mutable access.
    
    ApiResult::err(crate::error::ApiError::server_failure(
        "Data writing not implemented - requires mutable access to diagnostic system"
    ))
    
    // TODO: Implement with proper mutable access pattern:
    // 1. Use Arc<RwLock<Diagnostic>> in server setup
    // 2. Get write lock here
    // 3. Call write_data on the resource
    // 4. Return success or error response
}

