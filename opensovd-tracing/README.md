# opensovd-tracing

This crate provides conditional tracing macros for the OpenSOVD project. The macros allow
adding tracing throughout code that can be completely disabled at compile time when the
`tracing` feature is not enabled.

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `tracing` | No | Enables actual tracing functionality. When disabled, all macros compile to no-ops with zero overhead |

## Usage

Add this to the `Cargo.toml`:

```toml
[dependencies]
opensovd-tracing = { version = "0.0.1" }

[features]
default = ["tracing"]
tracing = ["opensovd-tracing/tracing"]
```

### Basic Logging Macros

The crate provides the standard logging macros that mirror the
[https://github.com/tokio-rs/tracing](tracing) crate:

```rust
use opensovd_tracing::{trace, debug, info, warn, error};

fn example() {
    trace!("This is a trace message");
    debug!("Debug information: {}", some_value);
    info!("Application started");
    warn!("This is a warning");
    error!("An error occurred: {}", error_msg);
}
```
