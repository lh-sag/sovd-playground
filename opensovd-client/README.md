# opensovd-client

OpenSOVD client library for communicating with SOVD diagnostic servers. Supports both HTTP and HTTPS connections
with optional mutual TLS (mTLS) authentication.

## Overview

This crate provides a client library for connecting to and interacting with OpenSOVD (ISO 17978) diagnostic servers.
It includes modern HTTP client functionality using hyper for making requests to SOVD servers and handling responses
with proper URL validation and parsing.

## Features

- Modern HTTP client using hyper (non-legacy) for SOVD server communication
- Proper URL parsing and validation using the `url` crate
- Version information retrieval
- Async/await support with Tokio
- Configurable timeouts and user agent
- Comprehensive error handling with derive_more
- JSON serialization/deserialization support
- **TLS/HTTPS support with OpenSSL**
- **Mutual TLS (mTLS) authentication**
- **Client certificate verification**
- **Custom CA certificate support**
- **Insecure mode for testing**

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | No | Enables tracing support via opensovd-tracing |
| `openssl` | No | Enables TLS/HTTPS support with mTLS authentication |

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

To enable TLS/mTLS support:

```toml
[dependencies]
opensovd-client = { version = "0.0.1", features = ["openssl"] }
```

To enable both tracing and TLS:

```toml
[dependencies]
opensovd-client = { version = "0.0.1", features = ["tracing", "openssl"] }
```

### Basic Example

```rust
use opensovd_client::Client;
use opensovd_models::version::VendorInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://localhost:9000".to_string());

    let version_info = client.version_info::<VendorInfo>().await?;
    println!("{:#?}", version_info);

    Ok(())
}
```

### With Custom Configuration

```rust
use opensovd_client::{Client, ClientConfig};
use std::time::Duration;

// Using ClientConfig directly
let config = ClientConfig::new("http://localhost:9000".to_string());
let client = Client::from_config(config)?; // Note: from_config returns Result

// Using ClientConfigBuilder
let client = Client::from_config(
    ClientConfig::builder("http://localhost:9000".to_string())
        .timeout(Duration::from_secs(10))
        .user_agent("my-app/1.0")
        .build()?
)?; // Note: from_config returns Result due to URL validation
```

### OpenSSL/HTTPS Examples

When the `openssl` feature is enabled, you can use HTTPS URLs and configure client certificates:

```rust
use opensovd_client::{Client, ClientConfig};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode, SslFiletype};

// Simple HTTPS connection (server cert verification only)
let client = Client::new("https://sovd-server.example.com:9001")?;

// HTTPS with custom CA certificate
let mut ssl_builder = SslConnector::builder(SslMethod::tls_client())?;
ssl_builder.set_ca_file("/path/to/ca.pem")?;
let ssl_connector = ssl_builder.build();

let client = Client::from_config(
    ClientConfig::builder("https://sovd-server.example.com:9001".to_string())
        .openssl(ssl_connector)
        .build()?
)?;

// Mutual TLS (mTLS) with client certificate
let mut ssl_builder = SslConnector::builder(SslMethod::tls_client())?;
ssl_builder.set_certificate_chain_file("/path/to/client.pem")?;
ssl_builder.set_private_key_file("/path/to/client.key", SslFiletype::PEM)?;
ssl_builder.set_ca_file("/path/to/ca.pem")?;
ssl_builder.set_verify(SslVerifyMode::PEER);
let ssl_connector = ssl_builder.build();

let client = Client::from_config(
    ClientConfig::builder("https://sovd-server.example.com:9001".to_string())
        .openssl(ssl_connector)
        .build()?
)?;

// Insecure mode (no certificate verification) - for testing only
let mut ssl_builder = SslConnector::builder(SslMethod::tls_client())?;
ssl_builder.set_verify(SslVerifyMode::NONE);
let ssl_connector = ssl_builder.build();

let client = Client::from_config(
    ClientConfig::builder("https://sovd-server.example.com:9001".to_string())
        .openssl(ssl_connector)
        .build()?
)?;
```

## API

### Client

The main client struct for communicating with SOVD servers.

#### Methods

- `new(url: String) -> Self` - Creates a new client instance with default configuration
- `from_config(config: ClientConfig) -> Result<Self, Error>` - Creates a client from a configuration (validates URL)
- `config(&self) -> &ClientConfig` - Returns the client configuration
- `version_info<T>() -> Result<VersionResponse<T>, Error>` - Retrieves version information from the server
- `url(&self) -> &str` - Returns the URL of the client

### ClientConfig

Configuration struct for the HTTP client.

#### Fields

- `url: String` - URL of the SOVD server
- `timeout: Duration` - Request and connection timeout (default: 30 seconds)
- `user_agent: String` - User-Agent header value (default: "opensovd-client/0.1.0")
- `ssl_connector: Option<SslConnector>` - OpenSSL connector for HTTPS connections (available with `openssl` feature)

### Client Methods

- `new(url: String) -> Self` - Creates configuration with defaults
- `validate(&self) -> Result<(), Error>` - Validates the URL in the configuration
- `builder(url: String) -> ClientConfigBuilder` - Creates a builder for fluent configuration

### ClientConfigBuilder

Builder for creating `ClientConfig` instances with fluent API.

### Builder Methods

- `new(url: String) -> Self` - Creates a new builder
- `timeout(timeout: Duration) -> Self` - Sets the timeout
- `user_agent<S: Into<String>>(user_agent: S) -> Self` - Sets the user agent
- `openssl(ssl_connector: SslConnector) -> Self` - Sets the OpenSSL connector for HTTPS connections
  (requires `openssl` feature)
- `build(self) -> Result<ClientConfig, ClientConfigError>` - Builds the final configuration with validation

### SslConnector

OpenSSL connector configuration for HTTPS connections (available with `openssl` feature).

Use `openssl::ssl::SslConnector::builder()` to create and configure SSL connections with:

- Client certificates for mTLS authentication
- Custom CA certificates for server verification
- Certificate verification settings
- Protocol and cipher configurations

See the OpenSSL documentation for complete configuration options.

### Error Handling

The crate provides a comprehensive `Error` enum that covers various error conditions:

- `Http(hyper::Error)` - HTTP transport errors
- `Uri(hyper::http::uri::InvalidUri)` - URI parsing errors
- `HttpError(hyper::http::Error)` - HTTP protocol errors
- `Json(serde_json::Error)` - JSON serialization/deserialization errors
- `UrlParse(url::ParseError)` - URL parsing errors
- `InvalidResponse(String)` - Invalid server responses or HTTP errors
- `Config(ClientConfigError)` - Configuration validation errors

#### ClientConfigError

Configuration-specific errors:

- `InvalidUrl(url::ParseError)` - URL parsing failed
- `MalformedUrl(String)` - URL validation failed (missing host, unsupported scheme, etc.)

## Implementation Details

This client uses modern hyper (v1.x) with manual connection handling instead of the legacy client. Features include:

- Proper URL parsing and validation before making requests
- Direct TCP connection management
- HTTP/1.1 support with proper connection lifecycle
- Configurable timeouts for both connection and request operations
- Proper error propagation using derive_more traits

## Dependencies

This crate depends on:

- `hyper` and `hyper-util` for HTTP client functionality
- `http-body-util` for HTTP body handling
- `url` for URL parsing and validation
- `urlencoding` for URL encoding utilities
- `tokio` for async runtime and TCP connections
- `serde` and `serde_json` for JSON handling
- `opensovd-models` for SOVD data structures
- `derive_more` for error handling conveniences
- `openssl` for TLS/SSL support (optional, with `openssl` feature)
- `tokio-openssl` for async OpenSSL integration (optional, with `openssl` feature)

## Security Considerations

When using OpenSSL/HTTPS features:

- **Certificate Verification**: Always use certificate verification in production (`verify_certs: true` is the default)
- **Certificate Storage**: Store private keys securely and with appropriate file permissions
- **Insecure Mode**: Only disable certificate verification for testing and development
- **Certificate Validation**: Ensure server certificates are valid and trusted
- **mTLS**: Use mutual TLS when the server requires client certificate authentication

### Example Certificate Setup

For mTLS, you'll typically need:

1. **CA Certificate** (`ca.pem`): The certificate authority that signed both client and server certificates
2. **Client Certificate** (`client.pem`): The client's certificate signed by the CA
3. **Client Private Key** (`client.key`): The client's private key corresponding to the certificate

Configure these using the OpenSSL SslConnector builder methods:

- `set_ca_file()` for CA certificates
- `set_certificate_chain_file()` for client certificates
- `set_private_key_file()` for private keys
- `set_verify()` for verification modes

Generate certificates using OpenSSL or your organization's PKI infrastructure.

## Integration

This client library is designed to be used by the `opensovd-cli` crate and other applications that need to communicate
with OpenSOVD servers. It provides a clean, async interface for SOVD protocol operations with modern HTTP client
implementation.
