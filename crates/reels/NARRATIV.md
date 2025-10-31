# Reels Backend Functionality Summary

This document provides a high-level overview of the functionalities implemented within the `crates/reels/backend` crate.

## Core Purpose

The backend serves as the primary API server for the Reels platform, built using the Actix-web framework. It handles user authentication, data persistence, business logic execution, and integration with various external services.

## Key Functional Areas & Features

1.  **API Routing (`src/routes/`)**: Defines all HTTP endpoints using Actix-web. Routes are organized by feature (e.g., auth, clone, creatives, collections, research, etc.).
    *   **Authentication (`src/routes/auth/`)**: Handles user registration, login, password reset, token verification, and logout. Uses JWT for session management.
    *   **Cloning (`src/routes/clone/`)**: Provides the core `/api/clone` endpoint for style replication, supporting "fast" (Zyte extraction) and "visual" (LLM-based) modes.
    *   **API Keys (`src/routes/api_keys/`)**: Manages API key creation, listing, and deletion for authenticated users.
    *   **Assets (`src/routes/assets/`)**: CRUD operations for user-uploaded assets, including GCS storage integration.
    *   **Collections (`src/routes/collections/`)**: CRUD operations for grouping creatives.
    *   **Creatives (`src/routes/creatives/`)**: Manages creative entities, including generation via LLM, Webflow integration (generation, listing, deletion), and standard CRUD.
    *   **Formats (`src/routes/formats/`)**: Manages public creative formats and user-defined custom creative formats.
    *   **Requests (`src/routes/requests/`)**: Tracks user requests (e.g., clone operations), storing metadata and status.
    *   **Research (`src/routes/research/`)**: CRUD endpoints for managing research tasks and results.
    *   **Styles (`src/routes/styles/`)**: CRUD operations for managing style references, including fetching content from URLs or using provided HTML, and storing links in GCS.
    *   **User DB Collections (`src/routes/user_db_collections/`)**: Manages user-defined database collections, including schema definition (potentially LLM-refined) and item CRUD operations with schema validation.
    *   **AgentLoop Integration (`src/routes/agentloop/` via `src/routes/mod.rs`)**: Integrates with the AgentLoop framework under the `/loupe` scope.
    *   **Stripe Webhooks (`src/routes/stripe_webhooks/`)**: Endpoint to handle incoming webhooks from Stripe.
    *   **Health Check (`src/routes/health.rs`)**: Basic `/health` endpoint.
    *   **Predefined Collections (`src/routes/predefined_collections`, `src/routes/user_db_collections`)**: Manages predefined-collections database collections, including schema definition (potentially LLM-refined), ui schema definition (potentially render the schema in UI i.e.: Forms, HTML or Link etc...) and item CRUD operations with schema validation.

2.  **Database Interaction (`src/db/`)**: Contains data models (structs mapping to tables) and functions for interacting with the PostgreSQL database using `sqlx`. Models cover users, requests, API keys, assets, collections, creatives, formats, research, styles, Webflow creatives, user DB collections/items, etc.

3.  **Authentication & Authorization (`src/auth/`, `src/middleware/auth.rs`)**:
    *   Handles JWT generation and verification (`src/auth/tokens.rs`).
    *   Provides password hashing (`bcrypt`).
    *   Implements `JwtMiddleware` for Actix-web to protect API routes, supporting both JWT and API key authentication.

4.  **External Service Integrations**:
    *   **Google Cloud Storage (`src/gcs.rs`)**: Client for uploading, downloading, and deleting objects (e.g., styles, assets, creative outputs).
    *   **Google Cloud Authentication (`src/gcp_auth.rs`)**: Handles authentication with GCP services.
    *   **LLM Integration (`src/llm/`)**: Unified interface (`src/llm/unified.rs`) to interact with different LLM vendors (e.g., Gemini, OpenAI). Includes typed LLM support (`src/llm_support/`).
    *   **Zyte (`src/zyte/`)**: Client for interacting with Zyte API for web data extraction (HTML, screenshots, inline styles).
    *   **Webflow (`src/webflow/`)**: Client for interacting with the Webflow CMS API (creating items, managing slugs).
    *   **Postmark (`src/email_service.rs`)**: Sends transactional emails (e.g., verification, password reset) via Postmark API.
    *   **Stripe (`src/routes/stripe_webhooks.rs`)**: Handles webhook events (specific logic likely within the handler).

5.  **Business Logic**:
    *   **Style Cloning/Replication (`src/style_cloning/`)**: Core logic for replicating website styles onto new content, potentially involving LLMs.
    *   **User Management (`src/user_management.rs`)**: Orchestrates user registration, login flows, and password reset initiation, coordinating DB and email services.
    *   **Utility Functions (`src/utils/`)**: Shared utilities like base64 processing for data URIs.

6.  **Configuration & Setup (`src/main.rs`, `src/openapi.rs`)**:
    *   Main application entry point, sets up Actix-web server, middleware, database pool, GCS client, AgentLoop state, logging (`env_logger`), CORS, and environment variables (`dotenvy`).
    *   Embeds frontend assets (`rust-embed`).
    *   Configures OpenAPI documentation using `utoipa`.

## Architectural Notes

*   Adheres (partially) to a strict "one item per file" guideline (`rust_coding_guidelines.md`), using fully qualified paths instead of `use` statements in many places.
*   Uses `sqlx` for async database operations with compile-time query checking.
*   Employs `anyhow` for error handling in some modules.
*   Uses `serde` for serialization/deserialization.
*   Integrates `utoipa` for automatic OpenAPI specification generation from code annotations.
