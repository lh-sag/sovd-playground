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
use opensovd_models::entity::{ComponentCapabilitiesResponse, EntityId, EntityReference, EntityResponse};

use crate::response::ApiResult;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/components", web::get().to(list_entities))
        .route("/components/{id}", web::get().to(list_capabilities));
    // TODO
    // GET /components/{component-id}/
    // GET /components/{component-id}/subcomponents
    // GET /components/{component-id}/hosts
    // GET /components/{component-id}/depends-on
}

// GET /components
async fn list_entities(
    base_uri: web::Data<super::BaseUri>,
    diagnostic: web::Data<Diagnostic>,
) -> ApiResult<EntityResponse> {
    // Collect components from the diagnostic system
    let items: Vec<EntityReference> = diagnostic
        .components()
        .map(|component| {
            let id = component.id().clone();
            let name = component.name().to_string();
            let href = format!("{}/components/{}", base_uri.0, id);

            EntityReference {
                entity: EntityId {
                    id,
                    name,
                    ..Default::default()
                },
                href,
                ..Default::default()
            }
        })
        .collect();
    if items.is_empty() {
        return ApiResult::err(crate::error::ApiError::not_found("No components found".to_string()));
    }
    ApiResult::ok(EntityResponse { items })
}

async fn list_capabilities(
    base_uri: web::Data<super::BaseUri>,
    component: web::Path<(String,)>,
    diagnostic: web::Data<Diagnostic>,
) -> ApiResult<ComponentCapabilitiesResponse> {
    // Try to get the component from the diagnostic system
    let component_id = &component.0;

    match diagnostic.get_component(component_id) {
        Some(comp) => {
            // Check what resources the component has
            let mut resources = opensovd_models::entity::Resources::default();

            // Check if component has a data resource
            if comp.resource().has_data_resource() {
                resources.data = Some(format!("/opensovd/v1/components/{}/data", component_id));
                resources.data_list = Some(format!("/opensovd/v1/components/{}/data-lists", component_id));
            }

            ApiResult::ok(ComponentCapabilitiesResponse {
                entity: EntityId {
                    id: comp.id().clone(),
                    name: comp.name().to_string(),
                    ..Default::default()
                },
                resources,
                ..Default::default()
            })
        }
        None => ApiResult::err(crate::error::ApiError::not_found(format!(
            "Component '{component_id}' not found"
        ))),
    }
}
