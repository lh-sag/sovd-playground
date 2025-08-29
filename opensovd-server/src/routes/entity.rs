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
use opensovd_models::entity::{ComponentCapabilitiesResponse, EntityId, EntityReference, EntityResponse};

use crate::response::create_api_response;

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
    registry: web::Data<ComponentRegistry>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError> {
    // Collect components from the registry
    let component_ids = registry.list_component_ids().await;
    let mut items = Vec::new();
    
    for id in component_ids {
        if let Some(component) = registry.get_component(&id).await {
            let name = component.name().to_string();
            let href = format!("{}/components/{}", base_uri.0, id);

            items.push(EntityReference {
                entity: EntityId {
                    id: id.clone(),
                    name,
                    ..Default::default()
                },
                href,
                ..Default::default()
            });
        }
    }

    if items.is_empty() {
        return Err(crate::response::ApiError::not_found("No components found"));
    }

    Ok(create_api_response(EntityResponse { items }, &req))
}

async fn list_capabilities(
    _base_uri: web::Data<super::BaseUri>,
    component: web::Path<(String,)>,
    registry: web::Data<ComponentRegistry>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::response::ApiError> {
    // Try to get the component from the registry
    let component_id = &component.0;

    // Early return if component not found
    let comp = registry
        .get_component(component_id)
        .await
        .ok_or_else(|| crate::response::ApiError::not_found(format!("Component '{component_id}' not found")))?;

    // Check what resources the component has
    let mut resources = opensovd_models::entity::Resources::default();

    // All components with the new architecture have data resources
    resources.data = Some(format!("/opensovd/v1/components/{component_id}/data"));
    resources.data_list = Some(format!("/opensovd/v1/components/{component_id}/data-lists"));

    let response = ComponentCapabilitiesResponse {
        entity: EntityId {
            id: comp.id().clone(),
            name: comp.name().to_string(),
            ..Default::default()
        },
        resources,
        ..Default::default()
    };

    Ok(create_api_response(response, &req))
}
