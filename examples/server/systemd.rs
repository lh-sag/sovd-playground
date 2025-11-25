// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Systemd socket activation example using sd-notify with named sockets
//
// Run with: systemd-socket-activate -l 9000 cargo run --example systemd-server
// Multiple named sockets: systemd-socket-activate -l 9000 -l 9001 --fdname=api:admin cargo run --example systemd-server

#[cfg(target_os = "linux")]
mod systemd {
    use std::os::unix::io::FromRawFd;
    use std::sync::Arc;

    use examples::{Component, EngineData};
    use sovd_diagnostic::{DiagnosticBuilder, data::DataService};
    use sovd_server::{Server, ServerConfig};

    #[tokio::main(flavor = "current_thread")]
    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        examples::init_logging();

        let diagnostic = DiagnosticBuilder::new()
            .with_entity(
                Component::new("engine".to_string(), "Engine Control Unit".to_string()),
                |ctx| ctx.with_service(Arc::new(EngineData) as Arc<dyn DataService>),
            )
            .build();

        let fds: Vec<(i32, String)> = sd_notify::listen_fds_with_names(false)?.collect();
        if fds.is_empty() {
            return Err("No file descriptors received from systemd".into());
        }

        tracing::info!("Starting SOVD server with systemd socket activation");

        let mut config_builder = ServerConfig::builder().diagnostic(diagnostic);

        for (i, (fd, name)) in fds.iter().enumerate() {
            let base_path = if name.is_empty() {
                format!("/sovd{i}")
            } else {
                format!("/{name}")
            };

            tracing::info!(num = i, fd = fd, name = ?name, base_path = ?base_path, "Socket configured for activation");

            let listener = unsafe { std::net::TcpListener::from_raw_fd(*fd) };
            config_builder = config_builder.endpoint(sovd_server::Listener::Tcp(listener), None, vec![], base_path);
        }

        let config = config_builder.build()?;

        Server::new(config).start().await?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    systemd::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This example is only available on Linux with systemd");
}
