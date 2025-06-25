<!-- markdownlint-disable -->
<h2 align="center">
    OpenSOVD Core
</h2>

<div align="center">

[![CI](https://github.com/sagovic/diag/workflows/CI/badge.svg)](https://github.com/sagovic/diag/actions/workflows/ci.yaml?query=branch%3Amain++)
[![slack](https://img.shields.io/badge/chat-slack-blue.svg?logo=slack)](https://app.slack.com/client/T02MS1M89UH/C0958MQNGP2)

</div>
<!-- markdownlint-enable -->

## Workspace Overview

This cargo workspace contains the following crates and CLI tools for the OpenSOVD (ISO 17978) implementation:

| Crate | Type | Description |
|-------|------|-------------|
| [`sovd`](sovd/README.md) | Library | Core SOVD data structures and types |
| [`opensovd-client`](opensovd-client/README.md) | Library | Client library for SOVD services |
| [`opensovd-server`](opensovd-server/README.md) | Library | HTTP server with REST API endpoints |
| [`opensovd-tracing`](opensovd-tracing/README.md) | Library | Conditional tracing with zero-cost abstractions |
| [`opensovd-cli`](opensovd-cli/README.md) | Binary Crate | Command-line tools and utilities |

### CLI Tools

The `opensovd-cli` crate provides the following command-line tools:

| Binary | Description |
|--------|-------------|
| `osovd-gateway` | OpenSOVD daemon/gateway server |
| `osovd-cli` | OpenSOVD CLI client for interacting with SOVD services |
