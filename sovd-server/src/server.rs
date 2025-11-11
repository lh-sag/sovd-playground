// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use actix_web::{App, HttpServer, guard, web};
use sovd_models::version::VendorInfo;
use tracing::info;

use crate::error::{Error, Result};
use crate::middleware::{auth::BearerAuth, tracing::Tracing};
use crate::routes;
use crate::server_config::{AuthInfo, Listener, ServerConfig};

const TARGET: &str = "srv";

pub struct Server<T = VendorInfo> {
    config: ServerConfig<T>,
}

fn configure<T>(
    cfg: &mut web::ServiceConfig,
    base: &str,
    server_name: &[String],
    vendor_info: Option<T>,
    diagnostic: Arc<sovd_diagnostic::Diagnostic>,
    auth: Option<BearerAuth>,
) where
    T: sovd_models::JsonSchema + Clone + Send + Sync + Default + 'static,
{
    let vendor_info = Some(vendor_info.unwrap_or_default());

    let mut scope = web::scope(base)
        .app_data(web::Data::new(routes::BaseUri(base.to_string())))
        .app_data(web::Data::new(vendor_info))
        .app_data(web::Data::from(diagnostic));

    // Add hostname guards if server_name is specified
    if !server_name.is_empty() {
        let first_guard = guard::Host(server_name[0].clone());
        let combined_guard = server_name[1..].iter().fold(guard::Any(first_guard), |acc, hostname| {
            acc.or(guard::Host(hostname.clone()))
        });
        scope = scope.guard(combined_guard);
    }

    let scope = scope.configure(routes::version::configure::<T>).service(
        web::scope("v1")
            .configure(routes::root::configure)
            .configure(routes::discovery::configure)
            .configure(routes::data::configure),
    );

    // Apply auth middleware if provided and add to config
    if let Some(auth_middleware) = auth {
        cfg.service(scope.wrap(auth_middleware));
    } else {
        cfg.service(scope);
    }
}

impl<T> Server<T>
where
    T: sovd_models::JsonSchema + Clone + Send + Sync + Default + 'static,
{
    pub fn new(config: ServerConfig<T>) -> Self {
        Self { config }
    }

    pub async fn start(self) -> Result<()> {
        info!(target: TARGET, "Start SOVD server");

        let endpoints = self.config.endpoints;
        let vendor_info = self.config.vendor_info.clone();
        let diagnostic = self.config.diagnostic.clone();

        // First pass: validate all auth configurations upfront (fail fast)
        for (idx, endpoint) in endpoints.iter().enumerate() {
            if let Some(auth_info) = &endpoint.auth {
                BearerAuth::from_rsa_pem(&auth_info.public_key_pem).map_err(|err| {
                    Error::AuthSetupFailed(format!("Failed to create bearer auth for endpoint {idx}: {err}"))
                })?;
            }
        }

        // Clone endpoint data for the HttpServer closure
        let endpoint_configs: Vec<(String, Vec<String>, Option<AuthInfo>)> = endpoints
            .iter()
            .map(|e| (e.base.clone(), e.server_name.clone(), e.auth.clone()))
            .collect();

        let server_builder = HttpServer::new(move || {
            let mut app = App::new().wrap(Tracing::new());
            for (base, server_name, auth_info) in &endpoint_configs {
                // Create BearerAuth - safe to unwrap because we validated above
                let auth = auth_info.as_ref().map(|info| {
                    BearerAuth::from_rsa_pem(&info.public_key_pem)
                        .expect("BearerAuth creation should succeed - was validated earlier")
                });

                app = app.configure(|cfg| {
                    configure::<T>(cfg, base, server_name, vendor_info.clone(), diagnostic.clone(), auth);
                });
            }

            #[cfg(feature = "ui")]
            let app = app.configure(routes::ui::configure);
            app
        });

        // Add listeners from the same endpoints
        let mut server_builder = server_builder;
        for endpoint in endpoints {
            server_builder = match endpoint.listener {
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
mod tests {
    use std::sync::Arc;

    use actix_web::{App, test};
    use sovd_diagnostic::Diagnostic;

    use super::*;

    #[actix_web::test]
    async fn test_server_timeout() {
        use std::net::TcpListener;

        use tokio::time::{Duration, timeout};

        let shutdown = async {};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = ServerConfig::builder()
            .endpoint(crate::Listener::Tcp(listener), None, vec![], "/test".to_string())
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

        let socket_addr = SocketAddr::from_abstract_name("test_sovd_server").unwrap();
        let listener = UnixListener::bind_addr(&socket_addr).unwrap();
        let config = ServerConfig::builder()
            .endpoint(crate::Listener::Unix(listener), None, vec![], "/test".to_string())
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
                .configure(|cfg| {
                    configure::<VendorInfo>(
                        cfg,
                        "/sovd",
                        &["localhost".to_string()],
                        Some(VendorInfo {
                            version: "1.0.0".to_string(),
                            name: "Test".to_string(),
                        }),
                        Arc::new(Diagnostic::empty()),
                        None,
                    )
                }),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/sovd/version-info")
            .insert_header(("content-type", "application/json"))
            .insert_header(("host", "localhost"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Should succeed with correct content-type");
    }
}
