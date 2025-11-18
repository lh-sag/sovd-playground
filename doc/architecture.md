# Architecture

## Component Overview

- **sovd-models** - ISO 17978-3 data structures and types
- **sovd-server** - REST endpoint implementation with HTTP/HTTPS/Unix-socket support
- **sovd-diagnostic** - Entity management and diagnostic services
- **sovd-cli** - Gateway binary with CLI interface (reference implementation)
- **sovd-ui** - Vue.js based web UI dashboard

## System Diagram

```mermaid
flowchart TB
    subgraph Clients
        Browser[Web Browser<br/>SOVD Dashboard]
        CLI[curl/HTTP Client]
    end

    subgraph Gateway["sovd-cli (Gateway Binary)"]
        Entry[CLI Parser<br/>Config Loader]
    end

    subgraph Server["sovd-server (HTTP Layer)"]
        HTTP[HTTP/HTTPS/Unix Socket<br/>Actix Web]
        Routes[REST Endpoints<br/>/v1/components, /v1/faults, etc.]
        UI[Web UI<br/>Vue.js Dashboard]
        Auth[Authentication<br/>JWT Optional]
    end

    subgraph Core["sovd-diagnostic (Business Logic)"]
        Entities[Entity Tree<br/>Areas/Components/Apps]
        Resources[Resource Collections<br/>Data/Faults/Operations]
        Services[Diagnostic Services<br/>UDS Integration]
    end

    subgraph Models["sovd-models"]
        Types[ISO 17978-3 Types<br/>Component/Fault/Data]
        Serialization[JSON Serialization<br/>OpenAPI Schema]
    end

    Browser -->|HTTPS/HTTP| HTTP
    Browser -->|/ui| UI
    CLI -->|HTTPS/HTTP| HTTP
    Entry --> HTTP
    HTTP --> Routes
    HTTP --> UI
    Routes --> Auth
    UI --> Routes
    Auth --> Entities
    Entities --> Resources
    Resources --> Services
    Services --> Types
    Routes --> Types
    Entities --> Serialization
```
