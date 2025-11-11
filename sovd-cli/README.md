# sovd-cli

Command-line tools for SOVD Playground. Provides the gateway server for ISO 17978 SOVD implementation.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `jsonschema-schemars` | Yes | JSON schema support for data validation |
| `openssl` | Yes | TLS/HTTPS support with mTLS |
| `ui` | Yes | Web UI for the gateway server |

## Usage

Build and run the gateway:

```bash
cargo build --package sovd-cli
cargo run --bin sovd-gateway
```

## Gateway Examples

Start with default configuration:

```bash
sovd-gateway
```

Start on specific URL:

```bash
sovd-gateway --url http://127.0.0.1:8080/sovd
```

Start with HTTPS:

```bash
sovd-gateway \
  --url https://127.0.0.1:8443/sovd \
  --cert server.pem \
  --key server.key \
  --cacert ca.pem
```

Use configuration file:

```bash
sovd-gateway --config config.toml
```
