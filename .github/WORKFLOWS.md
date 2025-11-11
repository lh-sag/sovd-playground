# GitHub Workflows

This directory contains the CI/CD automation for the SOVD Playground.

## Workflow Architecture

The CI/CD pipeline is split into focused, reusable workflows:

```mermaid
graph TD
    CI[ci.yaml<br/>on push to main] --> Build[build.yaml<br/>reusable workflow]
    Build --> PreCommit[Pre-commit checks]
    Build --> MultiOS[Multi-OS builds & tests]
    Build --> IntTests[integration-tests.yaml<br/>reusable workflow]
    CI --> Deploy[deploy.yaml<br/>creates 'latest' release]
    Deploy --> DockerTest[Test Docker image]
    Deploy --> Docker[Publish to GHCR]

    Nightly[nightly.yaml<br/>daily at 02:00 UTC] --> Build
    Nightly --> Deploy

    Release[release.yaml<br/>on push tags v*] --> Build
    Release --> Deploy

    Coverage[coverage.yaml<br/>after CI success] --> IntTests
    IntTests --> CovReport[Generate coverage report]
    CovReport --> Pages[GitHub Pages]
```

## Release tags

- `latest` - Latest successful main branch build
- `nightly` - Daily automated build at 02:00 UTC
- `vX.Y.Z` - Versioned production releases
