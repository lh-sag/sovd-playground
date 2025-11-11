// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
use actix_web::{HttpRequest, HttpResponse, web};
use sovd_models::version::VersionResponse;
use sovd_models::{IncludeSchemaQuery, JsonSchema};

use crate::response::create_api_response;

pub(crate) fn configure<T>(cfg: &mut web::ServiceConfig)
where
    T: JsonSchema + Clone + Send + Sync + 'static,
{
    cfg.service(web::resource("/version-info").route(web::get().to(get_version::<T>)));
}

/// GET  /version-info.
async fn get_version<T>(
    req: HttpRequest,
    base_uri: web::Data<super::BaseUri>,
    vendor_info: web::Data<Option<T>>,
    query: Result<web::Query<IncludeSchemaQuery>, actix_web::Error>,
) -> Result<HttpResponse, crate::response::ApiError>
where
    T: JsonSchema + Clone + Send + Sync + 'static,
{
    let query = query.unwrap_or_else(|_| web::Query(IncludeSchemaQuery::default()));
    let include_schema = query.include_schema;

    // Map ISO- to REST-API version.
    const VERSION: (&str, &str) = ("1.1", "v1");
    let response = VersionResponse {
        sovd_info: vec![sovd_models::version::Info {
            version: VERSION.0.to_string(),
            base_uri: format!("{}/{}", base_uri.resolve_uri(&req).trim_end_matches('/'), VERSION.1),
            vendor_info: vendor_info.as_ref().clone(),
        }],
    };

    Ok(create_api_response(response, include_schema))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test};

    use super::*;

    fn app<V>(
        base_uri: crate::routes::BaseUri,
        vendor_info: Option<V>,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    >
    where
        V: sovd_models::JsonSchema + Clone + Send + Sync + 'static,
    {
        App::new()
            .app_data(web::Data::new(base_uri))
            .app_data(web::Data::new(vendor_info))
            .configure(configure::<V>)
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
    struct TestVendorInfo {
        name: String,
        version: String,
    }

    #[actix_web::test]
    async fn test_get_version_success() {
        let base_uri = crate::routes::BaseUri("http://localhost".to_string());
        let vendor_info = Some(TestVendorInfo {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
        });

        let app = test::init_service(app(base_uri, vendor_info)).await;

        let req = test::TestRequest::get().uri("/version-info").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let version_resp: VersionResponse<TestVendorInfo> = serde_json::from_slice(&body).unwrap();

        assert_eq!(version_resp.sovd_info.len(), 1);
        assert_eq!(version_resp.sovd_info[0].version, "1.1");
        assert_eq!(version_resp.sovd_info[0].base_uri, "http://localhost:8080/v1");
    }

    #[actix_web::test]
    async fn test_get_version_with_schema() {
        let base_uri = crate::routes::BaseUri("http://localhost".to_string());
        let vendor_info = Some(TestVendorInfo {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
        });

        let app = test::init_service(app(base_uri, vendor_info)).await;

        let req = test::TestRequest::get()
            .uri("/version-info?include-schema=true")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // When include-schema=true, response should have schema field
        assert!(json.get("schema").is_some(), "Response should include schema");
        assert!(json.get("sovd_info").is_some(), "Response should include sovd_info");

        // Verify the schema is a valid JSON object
        let schema = json.get("schema").unwrap();
        assert!(schema.is_object(), "Schema should be a JSON object");
    }
}
