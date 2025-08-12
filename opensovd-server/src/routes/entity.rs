// Copyright (c) Contributors to the Eclipse Foundation
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
use actix_web::{Error, web};
use opensovd_models::IncludeSchemaParam;
use opensovd_models::entity::{ComponentCapabilitiesResponse, EntityId, EntityReference, EntityResponse};


pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/components", web::get().to(get_entities))
        .route("/components/{id}", web::get().to(get_entity_capabilities));
    // TODO
    // GET /components/{component-id}/
    // GET /components/{component-id}/subcomponents
    // GET /components/{component-id}/hosts
    // GET /components/{component-id}/depends-on
}

async fn get_entities(
    base_uri: web::Data<super::BaseUri>,
    include_schema: Result<web::Query<IncludeSchemaParam>, Error>,
) -> impl actix_web::Responder {
    let response = EntityResponse {
        items: vec![EntityReference {
            entity: EntityId {
                id: "DrivingComputer".to_string(),
                name: "Driving Computer".to_string(),
                ..Default::default()
            },
            href: format!("{}/components/{}", base_uri.0.to_string(), "DrivingComputer"),
            ..Default::default()
        }],
    };
    super::make_response(response, include_schema)
}

async fn get_entity_capabilities(
    _base_uri: web::Data<super::BaseUri>,
    component: web::Path<(String,)>,
    include_schema: Result<web::Query<IncludeSchemaParam>, Error>,
) -> impl actix_web::Responder {
    let response = ComponentCapabilitiesResponse {
        entity: EntityId {
            id: component.0.clone(),
            name: component.0.clone(),
            ..Default::default()
        },
        ..Default::default()
    };
    super::make_response(response, include_schema)
}
