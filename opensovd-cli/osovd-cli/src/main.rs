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

use clap::{Parser, Subcommand};
#[cfg(feature = "openssl")]
use libosovd::SslArgs;
use opensovd_client::Client;
use opensovd_client::ClientConfig;
use tracing::info;
#[cfg(feature = "openssl")]
use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(about = "OpenSOVD cli client")]
#[command(version = VERSION)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,

    /// SSL/TLS configuration options
    #[cfg(feature = "openssl")]
    #[command(flatten)]
    pub ssl: SslArgs,
}

#[derive(Subcommand)]
enum Commands {
    /// Get SOVD version information
    Version {
        /// Base URI of the SOVD server
        #[arg(env = "OSOVD_URL")]
        uri: String,
    },
    /// Interact with components
    Components {
        #[command(subcommand)]
        action: ComponentsAction,
    },
}

#[derive(Subcommand)]
enum ComponentsAction {
    /// List all components
    List {
        /// Base URI of the SOVD server
        #[arg(env = "OSOVD_URL")]
        uri: String,
    },
    /// Get specific component capabilities
    Get {
        /// Component ID
        id: String,
        /// Base URI of the SOVD server
        #[arg(env = "OSOVD_URL")]
        uri: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().compact().init();

    info!(version = VERSION, "OpenSOVD CLI client");

    let args = Args::parse();

    match args.command {
        Commands::Version { ref uri } => {
            let client = match create_client(&args, uri) {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Error creating client: {e}");
                    std::process::exit(1);
                }
            };

            match client.version_info::<opensovd_models::version::VendorInfo>().await {
                Ok(response) => {
                    let json = serde_json::to_string_pretty(&response)?;
                    println!("{json}");
                }
                Err(e) => {
                    eprintln!("Error getting version info: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Components { ref action } => {
            match action {
                ComponentsAction::List { uri } => {
                    let client = match create_client(&args, uri) {
                        Ok(client) => client,
                        Err(e) => {
                            eprintln!("Error creating client: {e}");
                            std::process::exit(1);
                        }
                    };
                    
                    match client.components().await {
                        Ok(response) => {
                            let json = serde_json::to_string_pretty(&response)?;
                            println!("{json}");
                        }
                        Err(e) => {
                            eprintln!("Error listing components: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                ComponentsAction::Get { id, uri } => {
                    let client = match create_client(&args, uri) {
                        Ok(client) => client,
                        Err(e) => {
                            eprintln!("Error creating client: {e}");
                            std::process::exit(1);
                        }
                    };
                    
                    match client.component_capabilities(id).await {
                        Ok(response) => {
                            let json = serde_json::to_string_pretty(&response)?;
                            println!("{json}");
                        }
                        Err(e) => {
                            eprintln!("Error getting component capabilities: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }

    info!("OpenSOVD CLI client finished successfully");
    Ok(())
}

fn create_client(args: &Args, uri: &str) -> Result<Client, Box<dyn std::error::Error>> {
    #[cfg(feature = "openssl")]
    {
        let mut config_builder = ClientConfig::builder(uri.to_string());

        // Configure OpenSSL if any options are provided
        if args.ssl.is_configured() {
            // Validate cert/key pair
            args.ssl.validate_cert_key_pair()?;

            let ssl_connector = build_client_ssl(&args.ssl)?;
            config_builder = config_builder.openssl(ssl_connector);
        }

        let config = config_builder.build()?;
        Ok(Client::from_config(config))
    }

    #[cfg(not(feature = "openssl"))]
    {
        // Check if OpenSSL options were provided but feature is disabled
        if false {
            // This condition will be optimized out but allows unused variable warning suppression
            let _ = args;
        }

        Client::new(uri.to_string()).map_err(|e| e.into())
    }
}

#[cfg(feature = "openssl")]
fn build_client_ssl(args: &SslArgs) -> Result<SslConnector, Box<dyn std::error::Error>> {
    let mut builder = SslConnector::builder(SslMethod::tls_client())?;

    // Configure certificate verification
    if args.insecure {
        builder.set_verify(SslVerifyMode::NONE);
    } else {
        builder.set_verify(SslVerifyMode::PEER);
    }

    // Load CA certificate if specified
    if let Some(ca_cert) = &args.cacert {
        builder.set_ca_file(ca_cert)?;
    }

    // Load client certificate and key for mTLS if specified
    if let (Some(cert), Some(key)) = (&args.cert, &args.key) {
        builder.set_certificate_chain_file(cert)?;
        builder.set_private_key_file(key, SslFiletype::PEM)?;
    }

    Ok(builder.build())
}
