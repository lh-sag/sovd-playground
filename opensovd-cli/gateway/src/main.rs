use clap::Command;
use opensovd_server::{Server, ServerConfig, ServerResult};
use opensovd_tracing::info;
use sovd::models::version::VendorInfo;
use std::net::TcpListener;
use tokio::runtime::Builder;

fn main() -> std::io::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

    info!(version = VERSION, "Starting OpenSOVD gateway server");

    let _matches = Command::new(env!("CARGO_BIN_NAME"))
        .about("OpenSOVD gateway daemon")
        .version(VERSION)
        .get_matches();

    let runtime = Builder::new_current_thread().enable_all().build()?;

    runtime
        .block_on(async_main())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

async fn async_main() -> ServerResult<()> {
    info!("Initializing OpenSOVD gateway server");

    // Create a TCP listener for the server
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    let addr = listener.local_addr()?;
    info!("Server will listen on {}", addr);

    // Configure the server
    let vendor_info = VendorInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "OpenSOVD Gateway".to_string(),
    };

    let config = ServerConfig::builder()
        .listen_address(listener)
        .vendor_info(vendor_info)
        .base_uri("http://127.0.0.1:9000/opensovd")?
        .build();

    // Create and start the server
    let server = Server::new(config);
    info!("Starting server on {}", addr);

    server.start().await
}
