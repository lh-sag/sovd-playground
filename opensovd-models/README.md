# opensovd-models

Core SOVD (ISO 17978) data structures and types.

## Overview

This crate provides the fundamental data structures and models for SOVD implementation.
All models are available at the top level of the crate.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
opensovd-models = "0.0.1"
```

## Examples

### Using version models

```rust
use opensovd_models::version::{Response, VendorInfo, Info};

// Create vendor information
let vendor_info = VendorInfo {
    version: "1.0.0".to_string(),
    name: "My Company".to_string(),
};

// Create version info
let version_info = Response {
    sovd_info: vec![Info {
        version: "1.1".to_string(),
        base_uri: "http://localhost:9000/v1".to_string(),
        vendor_info: Some(vendor_info),
    }],
};
```

### Using request/response models

```rust
use opensovd_models::{IncludeSchemaParam, ApiResponse};

// Create a request parameter
let param = IncludeSchemaParam {
    include_schema: true,
};

// Create a response with schema
let response = ApiResponse {
    data: version_info,
    schema: Some(serde_json::json!({"type": "object"})),
};
```
