<!-- markdownlint-disable -->
<h2 align="center">
    OpenSOVD Core - A diagnostic toolkit for automotive, construction, railway, and agricultural machinery
</h2>

<div align="center">

[![CI](https://github.com/sagovic/diag/workflows/CI/badge.svg)](https://github.com/sagovic/diag/actions/workflows/ci.yaml?query=branch%3Amain++)
[![Coverage](https://sagovic.github.io/diag/coverage-badge.svg)](https://sagovic.github.io/diag/)
[![slack](https://img.shields.io/badge/chat-slack-blue.svg?logo=slack)](https://app.slack.com/client/T02MS1M89UH/C0958MQNGP2)

</div>
<!-- markdownlint-enable -->

OpenSOVD Core is a Rust-based implementation of the ISO 17978 SOVD (Software Oriented Vehicle Diagnostics)
standard for diagnostics. It's part of the Eclipse Foundation's
[automotive](https://projects.eclipse.org/projects/automotive) initiatives.

## SOVD

SOVD represents a major shift from traditional automotive diagnostic protocols (like UDS) to modern RESTful APIs
for vehicle diagnostics. The standard enables unified access to both modern High Performance Computers (HPCs) and
classic ECUs through a single API.

## Features

- Multi-protocol support: HTTP, HTTPS, Unix sockets
- TLS/SSL support via OpenSSL
- Web UI for diagnostic visualization
- RESTful API for diagnostic data access
- Component-based diagnostics (engine, transmission controllers)
- Cross-platform (Linux, Windows, macOS)

## Workspace Overview

This cargo workspace contains the following crates and CLI tools for the OpenSOVD (ISO 17978) implementation:

| Crate | Type | Description |
|-------|------|-------------|
| [`opensovd-models`](opensovd-models/README.md) | Library | Core SOVD data structures and types |
| [`opensovd-client`](opensovd-client/README.md) | Library | Client library for SOVD services |
| [`opensovd-server`](opensovd-server/README.md) | Library | HTTP server with REST API endpoints |
| [`opensovd-tracing`](opensovd-tracing/README.md) | Library | Conditional tracing with zero-cost abstractions |
| [`opensovd-cli`](opensovd-cli/README.md) | Binary Crate | Command-line tools and utilities |
| [`opensovd-diagnostic`](opensovd-diagnostic/README.md) | Library | Diagnostic system components |

### CLI Tools

The `opensovd-cli` crate provides the following command-line tools:

| Binary | Description |
|--------|-------------|
| `osovd-gateway` | OpenSOVD gateway |
| `osovd-cli` | OpenSOVD CLI client for interacting with OpenSOVD gateway |
