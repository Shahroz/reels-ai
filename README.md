# bounti.ai Rust Monorepo
This repository contains the Rust-based services, libraries, and reference projects for bounti.ai. It is organized as a Cargo workspace and includes multiple crates and components, as outlined below.

## Directory structure 

```
.  
├── Cargo.toml            # Workspace manifest (agentloop, narrativ/backend)
├── crates/
│   ├── agentloop/        # AgentLoop service & library (CLI + frontend)
│   ├── narrativ/         # Narrativ service: React frontend + Rust backend
│   ├── llm/              # LLM abstraction library
│   ├── api_clients/      # Shared API client libraries
│   └── zino/             # Reference/demo project (not integrated)
├── build/                # Dockerfiles and build scripts (e.g., build/narrativ)
└── README.md             # This file
```

## Key crates

- **agentloop** (`crates/agentloop`)  
  Orchestrates autonomous agent loops. Includes:  
  - A command-line interface (`agentloop-cli`)  
  - A web frontend for monitoring

- **narrativ** (`crates/narrativ`)  
  Produces creative narratives using a full-stack service:  
  - React-based frontend (`crates/narrativ/frontend`)  
  - Actix Web backend (`crates/narrativ/backend`)

- **llm** (`crates/llm`)  
  Provides unified interfaces for multiple large-language-model providers.

## Reference projects

- **zino** (`crates/zino`)  
  Example/reference implementation kept for exploration; not part of the production code.

## Building & Docker

- Build the workspace with Cargo:  
  ```bash
  cargo build --workspace --release
  ```
- Dockerfiles for individual services are under `build/`, for example:  
  ```bash
  docker build -f build/narrativ/Dockerfile -t narrativ-service .
  docker run -p 8080:8080 --network=host --env-file .env narrativ-service
  ```

## Contributing

Please follow the coding guidelines in each crate (e.g., `rust_coding_guidelines.md`). Submit PRs against the `main` branch and ensure CI passes.
