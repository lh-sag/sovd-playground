# sovd-server

## Overview

This crate provides server-side functionality for the SOVD Playground (ISO 17978) implementation.

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `ui` | No | Serves UI files from the filesystem |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sovd-server = "0.0.1"
```

To enable embedded UI (for production deployments):

```toml
[dependencies]
sovd-server = { version = "0.0.1", features = ["ui"] }
```

### Basic Examples

#### Server with shutdown signal

```rust
use sovd_server::{Server, ServerConfig};
use std::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Shutdown signal received");
    };

    let config = ServerConfig::builder()
        .listen(listener)
        .shutdown(shutdown_signal)
        .build();

    let server = Server::new(config);
    server.start().await
}
```
