# opensovd-client

OpenSOVD client library for connecting to and interacting with SOVD services.

## Overview

This crate provides client-side functionality for the OpenSOVD (ISO 17978) implementation. Currently
contains minimal implementation that will be expanded to include client connection handling,
request/response processing, and service discovery.

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | No | Enables tracing support via opensovd-tracing |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
opensovd-client = "0.0.1"
```

To enable tracing:

```toml
[dependencies]
opensovd-client = { version = "0.0.1", features = ["tracing"] }
```
