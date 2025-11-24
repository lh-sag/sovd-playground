// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;
use std::process::ExitCode;

use clap::Parser;
use futures_util::FutureExt;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod, SslVerifyMode};
use serde::{Deserialize, Serialize};
use sovd_server::{AuthInfo, Server, ServerConfig};
use tokio::runtime::Builder;
use url::Url;

mod cli;
mod components;
mod config;
mod mock;

use libsovd::version::{ENABLED_FEATURES, VERSION};

const TARGET: &str = "gw";

fn create_listener(
    listen: &str,
    ssl_config: Option<&config::SslConfig>,
) -> Result<sovd_server::Listener, Box<dyn std::error::Error>> {
    use sovd_server::Listener;

    // Unix socket listener
    #[cfg(unix)]
    if let Some(unix_path) = listen.strip_prefix("unix:/") {
        let listener = if let Some(abstract_name) = unix_path.strip_prefix('@') {
            // Abstract socket (Linux only) - replace @ with null byte
            let abstract_path = format!("\0{abstract_name}");
            UnixListener::bind(&abstract_path)?
        } else {
            // Regular file-based socket
            UnixListener::bind(unix_path)?
        };
        return Ok(Listener::Unix(listener));
    }

    // TCP listener - bind to address
    let tcp_listener = TcpListener::bind(listen)?;

    // Check if SSL is configured
    if let Some(ssl_cfg) = ssl_config {
        #[cfg(feature = "openssl")]
        {
            let ssl = ssl_builder(
                ssl_cfg.cert.as_deref(),
                ssl_cfg.key.as_deref(),
                ssl_cfg.cacert.as_deref(),
                ssl_cfg.insecure,
                ssl_cfg.no_peer_cert,
            )?;
            return Ok(Listener::SecureTcp(tcp_listener, ssl));
        }
    }

    Ok(Listener::Tcp(tcp_listener))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema-schemars", derive(schemars::JsonSchema))]
struct OpenSovdInfo {
    version: String,
    name: String,
    features: Vec<String>,
}

#[allow(clippy::too_many_lines)]
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

    // Validate SSL configuration early
    #[cfg(feature = "openssl")]
    args.ssl.validate()?;

    let url = args.url.as_ref().map(|s| s.parse::<Url>()).transpose()?;
    let fallback_ssl = config::SslConfig::from(&args.ssl);

    // Build diagnostic with mock components
    let builder = sovd_diagnostic::DiagnosticBuilder::new();
    let builder = mock::create_mock_components(builder);
    let diagnostic = builder.build();

    // Create shared shutdown signal
    let shutdown_signal = async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => tracing::info!(target: TARGET, "Shutdown from Ctrl+C"),
            () = sigterm() => tracing::info!(target: TARGET, "Shutdown from SIGTERM"),
        }
    }
    .shared();

    // Subscribe to entity events
    let mut entity_events = diagnostic.entities().subscribe();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        let mut shutdown = shutdown_clone;
        loop {
            tokio::select! {
                event = entity_events.recv() => {
                    match event {
                        Ok(sovd_diagnostic::EntityEvent::Added { entity_id, entity }) => {
                            tracing::info!(target: TARGET, entity_id = %entity_id, entity_name = %entity.name(), "Entity added");
                        }
                        Ok(sovd_diagnostic::EntityEvent::Removed { entity_id, entity }) => {
                            tracing::info!(target: TARGET, entity_id = %entity_id, entity_name = %entity.name(), "Entity removed");
                        }
                        Err(_) => break,
                    }
                }
                () = &mut shutdown => {
                    tracing::info!(target: TARGET, "Entity event subscriber shutting down");
                    break;
                }
            }
        }
    });

    let mut servers = Vec::new();

    // Load config servers if config is provided
    if let Some(config_path) = &args.config {
        tracing::info!(target: TARGET, config_path = %config_path.display(), "Load configuration");
        let cfg = config::Config::from_file(config_path)
            .map_err(|e| format!("Failed to load configuration from {}: {e}", config_path.display()))?;
        servers.extend(cfg.servers);
    }

    // Add CLI server if --url is provided
    if let Some(url) = &url {
        #[cfg(unix)]
        let unix_listen = args.unix_socket.as_ref().map(|s| format!("unix:/{s}"));

        #[cfg(not(unix))]
        let unix_listen: Option<String> = None;

        let listen = if let Some(unix_socket_listen) = unix_listen {
            unix_socket_listen
        } else {
            let Some(host) = url.host_str() else {
                return Err("URL must have a host (e.g., http://localhost:9000/sovd)".into());
            };
            let default_port = match url.scheme() {
                "https" => 443,
                "http" => 80,
                scheme => return Err(format!("Unsupported URL scheme: {scheme}").into()),
            };
            let port = url.port().unwrap_or(default_port);
            format!("{host}:{port}")
        };

        servers.push(config::ServerConfig {
            name: "Command Line Server".into(),
            listen,
            server_name: vec![],
            base: url.path().into(),
            auth: args
                .auth_jwt
                .clone()
                .map(|jwt_public_key_path| config::AuthConfig { jwt_public_key_path }),
            ssl: (url.scheme() == "https").then(|| fallback_ssl.clone()),
        });
    }

    // If no config and no CLI option, use default
    if args.config.is_none() && url.is_none() {
        servers.extend(config::Config::default().servers);
    }

    let vendor_info = OpenSovdInfo {
        version: VERSION.into(),
        name: servers
            .first()
            .map_or_else(|| "SOVD Gateway".into(), |s| s.name.clone()),
        features: ENABLED_FEATURES.iter().map(ToString::to_string).collect(),
    };

    let mut config_builder = ServerConfig::builder_with_vendor_type::<OpenSovdInfo>()
        .vendor_info(vendor_info)
        .diagnostic(diagnostic);

    // Configure each server as an endpoint
    for server_cfg in &servers {
        // Create listener
        let listener = create_listener(&server_cfg.listen, server_cfg.ssl.as_ref())?;

        let actual_addr = listener.local_addr().unwrap_or_else(|| server_cfg.listen.clone());
        tracing::info!(
            target: TARGET,
            name = ?server_cfg.name,
            protocol = if server_cfg.ssl.is_some() { "https" } else { "http" },
            listening = ?actual_addr,
            base = ?server_cfg.base,
            "Add server"
        );

        // Prepare auth if configured
        let auth = if let Some(auth_cfg) = &server_cfg.auth {
            let public_key_pem = std::fs::read(&auth_cfg.jwt_public_key_path).map_err(|e| {
                format!(
                    "Failed to read JWT public key from {}: {e}",
                    auth_cfg.jwt_public_key_path
                )
            })?;
            tracing::info!(target: TARGET, server_name = %server_cfg.name, "Enable JWT authentication");
            Some(AuthInfo { public_key_pem })
        } else {
            None
        };

        // Add endpoint - no Result to unwrap!
        config_builder =
            config_builder.endpoint(listener, auth, server_cfg.server_name.clone(), server_cfg.base.clone());
    }

    // No need for separate command line URL handling - it's integrated above

    tracing::info!(target: TARGET, version = %cli::VERSION, features = ?ENABLED_FEATURES, "Start SOVD server");

    let config = config_builder.shutdown(shutdown_signal).build()?;
    Server::<OpenSovdInfo>::new(config).start().await?;
    Ok(())
}

#[cfg(feature = "openssl")]
fn ssl_builder(
    cert: Option<&str>,
    key: Option<&str>,
    cacert: Option<&str>,
    insecure: bool,
    no_peer_cert: bool,
) -> std::result::Result<SslAcceptorBuilder, Box<dyn std::error::Error>> {
    let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls_server())?;
    let key_path = key.ok_or("Private key file is required for HTTPS")?;
    let cert_path = cert.ok_or("Certificate file is required for HTTPS")?;

    builder.set_private_key_file(key_path, SslFiletype::PEM)?;
    builder.set_certificate_chain_file(cert_path)?;

    if let Some(ca) = cacert {
        builder.set_ca_file(ca)?;
    }
    let mut mode = SslVerifyMode::NONE;
    if !insecure {
        mode |= SslVerifyMode::PEER;
    }
    if !no_peer_cert {
        mode |= SslVerifyMode::FAIL_IF_NO_PEER_CERT;
    }
    builder.set_verify(mode);

    Ok(builder)
}

fn main() -> ExitCode {
    use tracing_subscriber::{EnvFilter, fmt};

    fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,actix_server=warn")))
        .compact()
        .init();
    match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => match runtime.block_on(serve()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Failed to serve: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("Failed to build runtime: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(unix)]
async fn sigterm() {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler")
        .recv()
        .await;
}

#[cfg(not(unix))]
async fn sigterm() {
    std::future::pending::<()>().await;
}
