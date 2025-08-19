# opensovd-cli

CLI tools package for OpenSOVD providing command-line utilities for interacting with SOVD services.
Supports both HTTP and HTTPS connections with optional mutual TLS (mTLS) authentication.

## Overview

This crate provides command-line tools for the OpenSOVD (ISO 17978) implementation, including:

- Diagnostic server for SOVD
- CLI client for interacting with SOVD servers

## Binaries

| Binary | Description |
|--------|-------------|
| `osovd-gateway` | OpenSOVD server for handling SOVD service requests |
| `osovd-cli` | OpenSOVD CLI client for interacting with SOVD services and topology management |

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | Yes | Enables tracing support with journald integration and formatted output |
| `openssl` | Yes | Enables TLS/HTTPS support with mTLS authentication for both client and server |
| `ui` | No | Enables web UI for the gateway server |
| `jsonschema-schemars` | Yes | Enables JSON schema support for data validation |

## Usage

Build all binaries:

```bash
cargo build --package opensovd-cli
```

Run the gateway daemon:

```bash
cargo run --bin osovd-gateway
```

Run the CLI client:

```bash
cargo run --bin osovd-cli version <URI>
```

## Examples

### CLI Client Examples

Get version information from HTTP server:

```bash
osovd-cli version http://localhost:9000
```

Get version information from HTTPS server with mTLS:

```bash
osovd-cli version https://sovd-server.example.com:9001 \
  --cert client.pem \
  --key client.key \
  --cacert ca.pem
```

Get version information with insecure HTTPS (no cert verification):

```bash
osovd-cli version https://sovd-server.example.com:9001 --insecure
```

### Gateway Server Examples

Start gateway on default port:

```bash
osovd-gateway
```

Start gateway on specific port:

```bash
osovd-gateway --url http://127.0.0.1:8080/opensovd
```

Start gateway with HTTP and HTTPS listeners (mTLS):

```bash
osovd-gateway \
  --url http://127.0.0.1:8080/opensovd \
  --url https://127.0.0.1:8443/opensovd \
  --cert server.pem \
  --key server.key \
  --cacert ca.pem
```

Start gateway with HTTPS in insecure mode (no client cert required):

```bash
osovd-gateway \
  --url https://127.0.0.1:8443/opensovd \
  --cert server.pem \
  --key server.key \
  --insecure \
  --no-peer-cert
```

Start gateway with Unix socket:

```bash
osovd-gateway --url unix:///tmp/opensovd.sock
```

## TLS/mTLS Support

Both `osovd-cli` and `osovd-gateway` support OpenSSL/HTTPS connections with mutual TLS authentication:

### Client OpenSSL Options (osovd-cli)

| Option | Description |
|--------|-------------|
| `--cert <FILE>` | Path to client certificate file (PEM format) for mTLS |
| `--key <FILE>` | Path to client private key file (PEM format) for mTLS |
| `--cacert <FILE>` | CA certificate file (PEM format) for server verification |
| `--insecure` | Disable TLS certificate verification (insecure mode) |

### Server OpenSSL Options (osovd-gateway)

| Option | Description |
|--------|-------------|
| `--cert <FILE>` | Use the server certificate stored in file |
| `--key <FILE>` | Use the private key in file |
| `--cacert <FILE>` | Use the certificate authorities ("CA") to verify the peers |
| `--insecure` | No peer verification |
| `--no-peer-cert` | No peer certificate required |

### Security Considerations

- **Certificate Verification**: Always use certificate verification in production
- **Certificate Storage**: Store private keys securely with appropriate file permissions
- **Insecure Mode**: Only use `--insecure` for testing and development
- **mTLS**: Use mutual TLS when both client and server authentication is required

The client-side OpenSSL configuration is handled internally using OpenSSL's SslConnector,
which provides fine-grained control over certificates, protocols, and verification settings.

## Configuration

To disable default features:

```toml
[dependencies]
opensovd-cli = { version = "0.0.1", default-features = false }
```

To enable only specific features:

```toml
[dependencies]
opensovd-cli = { version = "0.0.1", default-features = false, features = ["openssl"] }
```
