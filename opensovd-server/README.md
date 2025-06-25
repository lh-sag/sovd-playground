# opensovd-server

HTTP server implementation for OpenSOVD with Actix-web and REST API endpoints.

## Overview

This crate provides server-side functionality for the OpenSOVD (ISO 17978) implementation, including:

- HTTP server with Actix-web framework
- REST API endpoints for SOVD services
- Asynchronous request handling
- JSON serialization/deserialization support

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | Yes | Enables tracing support via opensovd-tracing |
| `http2` | No | Enables HTTP/2 support in the Actix Web server |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
opensovd-server = "0.0.1"
```

To disable tracing:

```toml
[dependencies]
opensovd-server = { version = "0.0.1", default-features = false }
```

To enable HTTP/2 support:

```toml
[dependencies]
opensovd-server = { version = "0.0.1", features = ["http2"] }
```

To enable both tracing and HTTP/2:

```toml
[dependencies]
opensovd-server = { version = "0.0.1", features = ["tracing", "http2"] }
```

### Basic Examples

#### Server with shutdown signal

```rust
use opensovd_server::{Server, ServerConfig};
use std::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // Create shutdown signal (e.g., Ctrl+C)
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Shutdown signal received");
    };

    let config = ServerConfig::builder()
        .listen_address(listener)
        .shutdown(shutdown_signal)
        .build();

    let server = Server::new(config);
    server.start().await
}
```

#### Server without shutdown (runs indefinitely)

```rust
use opensovd_server::{Server, ServerConfig};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    let config = ServerConfig::builder()
        .listen_address(listener)
        .build();

    let server = Server::new(config);
    server.start().await
}
```

#### Unix Domain Socket Server

```rust
use opensovd_server::{Server, ServerConfig};
use std::os::unix::net::UnixListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = UnixListener::bind("/tmp/opensovd.sock")?;

    let config = ServerConfig::builder()
        .listen_uds(listener)
        .build();

    let server = Server::new(config);
    server.start().await
}
```

## API Endpoints

The server provides the following REST API endpoints:

- `GET /version-info` - Returns SOVD version information
- `GET /v1/version-info` - Versioned endpoint for SOVD version information

Both endpoints return JSON responses with the current SOVD version information.
