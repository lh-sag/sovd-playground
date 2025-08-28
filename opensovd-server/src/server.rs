//
// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
//

use actix_web::{App, HttpServer, guard, web};
use opensovd_models::version::VendorInfo;
use tracing::info;

use crate::error::{Error, Result};
use crate::middleware::{auth::BearerAuth, tracing::Tracing};
use crate::routes;
use crate::server_config::{Listener, ServerConfig};

const TARGET: &str = "srv";

/// The main OpenSOVD HTTP Server structure.
pub struct Server<T = VendorInfo> {
    config: ServerConfig<T>,
}

impl<T> Server<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + schemars::JsonSchema + Clone + Send + Sync + 'static,
{
    /// Creates a new server instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The server configuration including binding and optional shutdown.
    pub fn new(config: ServerConfig<T>) -> Self {
        Self { config }
    }

    /// Starts the HTTP server and binds to the configured address.
    ///
    /// This method will block the current thread until the server is shut down.
    /// To shut down the server gracefully, you can provide a shutdown signal
    /// when creating the server configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server shuts down gracefully, or an error if
    /// the server fails to start or encounters an error during operation.
    pub async fn start(self) -> Result<()> {
        info!(target: TARGET, "Starting OpenSOVD server");

        let uri_path = self.config.uri_path.trim_end_matches('/').to_string();
        let base_uri = self.config.uri_path.clone();
        let vendor_info = self.config.vendor_info.clone();
        let diagnostic = self.config.diagnostic.clone();

        // Setup authentication middleware if configured
        let auth_middleware = if let Some(auth_info) = self.config.auth {
            let bearer_auth = BearerAuth::from_rsa_pem(&auth_info.public_key_pem)
                .map_err(|e| Error::AuthSetupFailed(format!("Failed to create bearer auth: {}", e)))?;
            Some(bearer_auth)
        } else {
            None
        };

        let server_builder = HttpServer::new(move || {
            let app = App::new()
                .wrap(Tracing::new())
                .app_data(web::Data::new(routes::BaseUri(base_uri.clone())))
                .app_data(web::Data::new(vendor_info.clone()))
                .app_data(web::Data::from(diagnostic.clone()));

            let base_scope = web::scope(&uri_path)
                .guard(guard::Header("content-type", "application/json"))
                .configure(routes::version::configure::<T>);

            // Create v1 scope with conditional auth
            let app = if let Some(ref auth) = auth_middleware {
                let v1_scope = web::scope("v1")
                    .wrap(auth.clone())
                    .configure(routes::entity::configure)
                    .configure(routes::data::configure);
                app.service(base_scope.service(v1_scope))
            } else {
                let v1_scope = web::scope("v1")
                    .configure(routes::entity::configure)
                    .configure(routes::data::configure);
                app.service(base_scope.service(v1_scope))
            };
            #[cfg(feature = "ui")]
            let app = app.configure(routes::ui::configure);
            let app = app.configure(routes::metrics::configure);
            app
        });

        let mut server_builder = server_builder;
        for listener in self.config.listeners {
            server_builder = match listener {
                Listener::Tcp(tcp_listener) => server_builder.listen(tcp_listener)?,
                #[cfg(feature = "openssl")]
                Listener::SecureTcp(tcp_listener, ssl) => server_builder.listen_openssl(tcp_listener, ssl)?,
                #[cfg(unix)]
                Listener::Unix(unix_listener) => server_builder.listen_uds(unix_listener)?,
            };
        }

        let server = server_builder;

        let server = if let Some(shutdown) = self.config.shutdown {
            server.shutdown_signal(shutdown)
        } else {
            server
        };
        server.workers(1).run().await.map_err(Error::Io)
    }
}

#[cfg(test)]
fn configure<T>(cfg: &mut web::ServiceConfig, base_path: &str)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + schemars::JsonSchema + Clone + Send + Sync + 'static,
{
    cfg.service(
        web::scope(base_path)
            .guard(guard::Header("content-type", "application/json"))
            .configure(routes::version::configure::<T>)
            .configure(routes::hello::configure)
            .service(
                web::scope("v1")
                    .configure(routes::entity::configure)
                    .configure(routes::data::configure),
            ),
    );
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test};

    use super::*;

    #[actix_web::test]
    async fn test_server_timeout() {
        use std::net::TcpListener;

        use tokio::time::{Duration, timeout};

        let shutdown = async {};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder()
            .listen(listener)
            .shutdown(shutdown)
            .vendor_info(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            })
            .build()
            .unwrap();
        let server = Server::new(config);
        let result = timeout(Duration::from_secs(1), server.start()).await;
        assert!(result.is_ok(), "Server should shutdown when future is awaited");
    }

    #[actix_web::test]
    #[cfg(target_os = "linux")]
    async fn test_server_uds() {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixListener};

        use tokio::time::{Duration, timeout};

        let socket_addr = SocketAddr::from_abstract_name("test_opensovd_server").unwrap();
        let listener = UnixListener::bind_addr(&socket_addr).unwrap();
        let config = ServerConfig::builder()
            .listen_uds(listener)
            .shutdown(std::future::ready(()))
            .vendor_info(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            })
            .build()
            .unwrap();
        let server = Server::new(config);
        let result = timeout(Duration::from_secs(1), server.start()).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_guard_positive() {
        use crate::routes::BaseUri;

        let base_uri = BaseUri("/".to_string());
        let vendor_info: Option<VendorInfo> = Some(VendorInfo {
            version: "1.0.0".to_string(),
            name: "Test Vendor".to_string(),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(base_uri))
                .app_data(web::Data::new(vendor_info))
                .configure(|cfg| configure::<VendorInfo>(cfg, "/opensovd")),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/opensovd/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Should succeed with correct content-type");
    }

    #[actix_web::test]
    async fn test_guard_negative() {
        use crate::routes::BaseUri;

        let base_uri = BaseUri("/".to_string());
        let vendor_info: Option<VendorInfo> = Some(VendorInfo {
            version: "1.0.0".to_string(),
            name: "Test Vendor".to_string(),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(base_uri))
                .app_data(web::Data::new(vendor_info))
                .configure(|cfg| configure::<VendorInfo>(cfg, "/opensovd")),
        )
        .await;
        let req = test::TestRequest::get().uri("/opensovd/version-info").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::NOT_FOUND,
            "Should fail without correct content-type"
        );
    }

    #[actix_web::test]
    async fn test_guard_wrong_content_type() {
        let app = test::init_service(App::new().configure(|cfg| configure::<VendorInfo>(cfg, "/opensovd"))).await;
        let req = test::TestRequest::get()
            .uri("/opensovd/version-info")
            .insert_header(("content-type", "text/plain"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    #[cfg(feature = "jsonschema-schemars")]
    async fn test_custom_vendor_info() {
        use opensovd_models::version::VersionResponse;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
        struct CustomVendorInfo {
            build_date: String,
            features: Vec<String>,
        }

        let _custom_vendor = CustomVendorInfo {
            build_date: "2025-01-01".to_string(),
            features: vec!["tracing".to_string(), "http2".to_string()],
        };

        use crate::routes::BaseUri;

        let base_uri = BaseUri("/".to_string());
        let vendor_info: Option<CustomVendorInfo> = Some(_custom_vendor);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(base_uri))
                .app_data(web::Data::new(vendor_info))
                .configure(|cfg| configure::<CustomVendorInfo>(cfg, "/opensovd")),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/opensovd/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success(), "Should succeed with custom vendor info");

        // Parse response body to verify custom data
        let body = test::read_body(resp).await;
        let response: VersionResponse<CustomVendorInfo> = serde_json::from_slice(&body).unwrap();

        assert_eq!(response.sovd_info.len(), 2);
        assert_eq!(response.sovd_info[0].version, "1.1");
        assert_eq!(response.sovd_info[0].base_uri, "/v1");

        let vendor = response.sovd_info[0].vendor_info.as_ref().unwrap();
        assert_eq!(vendor.build_date, "2025-01-01");
        assert_eq!(vendor.features, vec!["tracing", "http2"]);
    }

    #[actix_web::test]
    async fn test_custom_base_path() {
        use crate::routes::BaseUri;

        let base_uri = BaseUri("/".to_string());
        let vendor_info: Option<VendorInfo> = Some(VendorInfo {
            version: "1.0.0".to_string(),
            name: "Test Vendor".to_string(),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(base_uri))
                .app_data(web::Data::new(vendor_info))
                .configure(|cfg| configure::<VendorInfo>(cfg, "/api/v2/opensovd")),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v2/opensovd/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Should succeed with custom base path");

        let req = test::TestRequest::get()
            .uri("/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    // Temporarily disabled due to tokio::spawn Send trait issues
    // #[cfg(unix)]
    // #[actix_web::test]
    // async fn test_server_shutdown_on_ctrl_c() {
    //     use std::net::TcpListener;
    //     use tokio::time::{Duration, sleep, timeout};
    //     let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    //     let shutdown_signal = async move {
    //         tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    //     };
    //     let config = ServerConfig::builder()
    //         .listen(listener)
    //         .vendor_info(VendorInfo {
    //             version: env!("CARGO_PKG_VERSION").to_string(),
    //             name: "Test Server".to_string(),
    //         })
    //         .shutdown(shutdown_signal)
    //         .build()
    //         .unwrap();
    //     let server = Server::new(config);
    //     let server_handle = tokio::spawn(async move { server.start().await });
    //     sleep(Duration::from_millis(100)).await;
    //     unsafe {
    //         libc::kill(libc::getpid(), libc::SIGINT);
    //     }
    //     let result = timeout(Duration::from_secs(5), server_handle).await;
    //     assert!(result.is_ok(), "Server should shut down within timeout");
    //     let server_result = result.unwrap();
    //     assert!(server_result.is_ok(), "Server task should complete successfully");
    //     assert!(server_result.unwrap().is_ok(), "Server should shut down cleanly");
    // }
}
