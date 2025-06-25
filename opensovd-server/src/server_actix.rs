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

#[cfg(feature = "ui")]
use actix_files::Files;
use actix_web::{App, HttpServer, guard, web};

use crate::error::{ServerError, ServerResult};
use crate::server::{ServerConfig, Socket};
use http::Uri;
use opensovd_tracing::info;
use sovd::models::version::{VendorInfo, VersionInfo};
use std::net::TcpListener;

/// Shared state for the OpenSOVD HTTP Server.
#[derive(Debug, Clone)]
pub struct ServerState<T = VendorInfo> {
    /// The vendor information.
    pub vendor_info: T,
    /// The base URI for the server.
    pub base_uri: Uri,
}

/// The main OpenSOVD HTTP Server structure.
pub struct Server<T = VendorInfo> {
    config: ServerConfig<T>,
    state: ServerState<T>,
}

impl<T> Server<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    /// Creates a new server instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The server configuration including binding and optional shutdown.
    pub fn new(config: ServerConfig<T>) -> Self {
        info!("Creating new OpenSOVD server");

        let state = ServerState {
            vendor_info: config.vendor_info.clone(),
            base_uri: config.base_uri.clone(),
        };
        Self { config, state }
    }

    /// Configures the Actix App routes and guards using ServiceConfig.
    fn configure_app(cfg: &mut web::ServiceConfig, base_path: &str) {
        cfg.service(
            web::scope(base_path)
                .guard(guard::Header("content-type", "application/json"))
                .route("/version-info", web::get().to(get_version_info::<T>))
                .service(web::scope("/v1").route("/version-info", web::get().to(get_version_info::<T>))),
        );
    }

    #[cfg(feature = "ui")]
    fn configure_ui(cfg: &mut web::ServiceConfig) {
        cfg.service(Files::new("/ui", "./assets").index_file("index.html"));
    }

    /// Starts the HTTP server and binds to the configured address.
    ///
    /// This function will run until the shutdown future completes.
    pub async fn start(self) -> ServerResult<()> {
        info!("Starting OpenSOVD server");

        let state = self.state.clone();
        let base_path = self.config.base_uri.path().to_string();
        let server_builder = HttpServer::new(move || {
            let app = App::new()
                .app_data(web::Data::new(state.clone()))
                .configure(|cfg| Self::configure_app(cfg, &base_path));
            #[cfg(feature = "ui")]
            let app = app.configure(Self::configure_ui);
            app
        });

        let server = match self.config.socket() {
            Some(Socket::TcpListener(listener)) => server_builder.listen(listener.try_clone()?)?,
            #[cfg(unix)]
            Some(Socket::UnixSocket(listener)) => server_builder.listen_uds(listener.try_clone()?)?,
            None => {
                // Default to binding on localhost with a random port
                let listener = TcpListener::bind("127.0.0.1:0")?;
                server_builder.listen(listener)?
            }
        };

        let server = if let Some(shutdown) = self.config.shutdown {
            server.shutdown_signal(shutdown)
        } else {
            server
        };
        server.run().await.map_err(ServerError::Io)
    }
}

/// Handles GET requests for `/version-info`.
///
/// This endpoint returns the current SOVD version information as a JSON object.
async fn get_version_info<T>(state: web::Data<ServerState<T>>) -> impl actix_web::Responder
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    info!("Handling GET /version-info request");
    const ISO_VERSION: &str = "1.1";
    // Construct VersionInfo with the vendor info
    let version_info = VersionInfo {
        info: vec![sovd::models::version::Info {
            version: ISO_VERSION.to_string(),
            base_uri: state.base_uri.to_string(),
            vendor_info: Some(state.vendor_info.clone()),
        }],
    };
    web::Json(version_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::DEFAULT_BASE_URI;

    use tokio::time::{Duration, timeout};

    #[actix_web::test]
    async fn test_server_timeout() {
        use std::net::TcpListener;

        // Use TCP listener
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder()
            .listen_address(listener)
            .shutdown(std::future::ready(()))
            .vendor_info(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            })
            .build();
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

        // Use abstract Unix domain socket
        let socket_addr = SocketAddr::from_abstract_name("test_opensovd_server").unwrap();
        let listener = UnixListener::bind_addr(&socket_addr).unwrap();
        let config = ServerConfig::builder()
            .listen_uds(listener)
            .shutdown(std::future::ready(()))
            .vendor_info(VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            })
            .build();
        let server = Server::new(config);
        let result = timeout(Duration::from_secs(1), server.start()).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_guard_positive() {
        use actix_web::{App, test, web};
        let state = ServerState {
            vendor_info: VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            },
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .configure(|cfg| Server::<VendorInfo>::configure_app(cfg, "/opensovd")),
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
        use actix_web::{App, test, web};
        let state = ServerState {
            vendor_info: VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            },
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .configure(|cfg| Server::<VendorInfo>::configure_app(cfg, "/opensovd")),
        )
        .await;

        let req = test::TestRequest::get().uri("/opensovd/version-info").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::NOT_FOUND,
            "Should return 404 without content-type"
        );
    }

    #[actix_web::test]
    async fn test_guard_wrong_content_type() {
        use actix_web::{App, test, web};
        let state = ServerState {
            vendor_info: VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            },
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .configure(|cfg| Server::<VendorInfo>::configure_app(cfg, "/opensovd")),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/opensovd/version-info")
            .insert_header(("content-type", "text/plain"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_custom_vendor_info() {
        use actix_web::{App, test, web};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct CustomVendorInfo {
            build_date: String,
            features: Vec<String>,
        }

        let custom_vendor = CustomVendorInfo {
            build_date: "2025-01-01".to_string(),
            features: vec!["tracing".to_string(), "http2".to_string()],
        };

        let state = ServerState {
            vendor_info: custom_vendor,
            base_uri: DEFAULT_BASE_URI
                .parse::<Uri>()
                .expect("DEFAULT_BASE_URI should be valid"),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(|cfg| Server::<CustomVendorInfo>::configure_app(cfg, "/opensovd")),
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
        let response: VersionInfo<CustomVendorInfo> = serde_json::from_slice(&body).unwrap();

        assert_eq!(response.info.len(), 1);
        assert_eq!(response.info[0].version, "1.1");
        assert_eq!(response.info[0].base_uri, DEFAULT_BASE_URI);

        let vendor = response.info[0].vendor_info.as_ref().unwrap();
        assert_eq!(vendor.build_date, "2025-01-01");
        assert_eq!(vendor.features, vec!["tracing", "http2"]);
    }

    #[actix_web::test]
    async fn test_custom_base_path() {
        use actix_web::{App, test, web};

        // Test with custom base path
        let state = ServerState {
            vendor_info: VendorInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "Test Server".to_string(),
            },
            base_uri: "/api/v2/opensovd".parse::<Uri>().expect("should be valid"),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(|cfg| Server::<VendorInfo>::configure_app(cfg, "/api/v2/opensovd")),
        )
        .await;

        // Test that the endpoint is available at the custom path
        let req = test::TestRequest::get()
            .uri("/api/v2/opensovd/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Should succeed with custom base path");

        // Test that the old path doesn't work
        let req = test::TestRequest::get()
            .uri("/version-info")
            .insert_header(("content-type", "application/json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_server_with_no_socket_config() {
        use tokio::time::{Duration, timeout};

        // Test that server can start without socket configuration
        let vendor_info = VendorInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            name: "Test Server".to_string(),
        };

        let config = ServerConfig::builder()
            .vendor_info(vendor_info)
            .shutdown(std::future::ready(()))
            .build();

        // Verify no socket is configured
        assert!(config.socket().is_none());

        let server = Server::new(config);
        let result = timeout(Duration::from_secs(1), server.start()).await;
        assert!(
            result.is_ok(),
            "Server should start successfully with no socket config and use default"
        );
    }
}
