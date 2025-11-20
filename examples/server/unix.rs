// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//
// Unix domain socket server example (abstract and file-based)
//
// Run with: cargo run --example unix-server
//
// Abstract unix socket: curl --silent --show-error --abstract-unix-socket sovd http://localhost/local1/version-info | jq
// Unix socket: curl --silent --show-error --unix-socket /tmp/sovd.sock http://localhost/local2/version-info | jq

#[cfg(unix)]
mod unix {
    use std::os::unix::net::UnixListener;
    use std::sync::Arc;

    use examples::{Ecu, EngineData};
    use sovd_diagnostic::{DiagnosticBuilder, data::DataService};
    use sovd_server::{Server, ServerConfig};

    #[tokio::main(flavor = "current_thread")]
    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        examples::init_logging();

        let diagnostic = DiagnosticBuilder::new()
            .with_entity(
                Ecu::new("engine".to_string(), "Engine Control Unit".to_string()),
                |ctx| ctx.with_service(Arc::new(EngineData) as Arc<dyn DataService>),
            )
            .build()?;

        // Start SOVD server on Unix domain sockets
        let socket_path = "/tmp/sovd.sock";
        let _ = std::fs::remove_file(socket_path);
        let file_listener = UnixListener::bind(socket_path)?;
        tracing::info!("Starting SOVD server on Unix socket {}", socket_path);

        let mut config = ServerConfig::builder().diagnostic(Arc::new(diagnostic)).endpoint(
            sovd_server::Listener::Unix(file_listener),
            None,
            vec![],
            "/local2".into(),
        );

        #[cfg(target_os = "linux")]
        {
            use std::os::linux::net::SocketAddrExt;
            use std::os::unix::net::SocketAddr;

            let socket_addr = SocketAddr::from_abstract_name("sovd")?;
            let abstract_listener = UnixListener::bind_addr(&socket_addr)?;
            tracing::info!("Starting SOVD server on abstract Unix socket @sovd");
            config = config.endpoint(
                sovd_server::Listener::Unix(abstract_listener),
                None,
                vec![],
                "/local1".into(),
            );
        }

        let config = config.build()?;
        Server::new(config).start().await?;
        Ok(())
    }
}

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unix::main()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("This example is only available on Unix platforms");
}
