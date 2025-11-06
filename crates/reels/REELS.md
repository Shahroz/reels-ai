# Reels Backend Functionality Summary

This document provides a high-level overview of the functionalities implemented within the `crates/reels/backend` crate.

## Core Purpose

The backend serves as a lightweight API server wrapper around the AgentLoop framework, built using Actix-web. It provides Reels-specific agent tools (reel generation and web browsing) integrated into AgentLoop's session-based research and conversation system. The backend handles service orchestration, external integrations, and exposes AgentLoop's capabilities under the `/loupe` scope.

## Key Functional Areas & Features

### 1. API Routing (`src/routes/`)

The routing layer has been significantly simplified. Most traditional route folders have been removed in favor of AgentLoop integration.

*   **Health Check (`src/routes/health.rs`)**: Basic `/health` endpoint that returns server status.
*   **AgentLoop Integration (`src/routes/mod.rs`)**: Mounts AgentLoop routes under `/loupe` scope, providing:
    *   `/loupe/research` - Initiates research tasks
    *   `/loupe/session/{session_id}/status` - Gets session status
    *   `/loupe/session/{session_id}/message` - Posts messages to sessions
    *   `/loupe/session/{session_id}/terminate` - Terminates sessions
    *   `/loupe/session/{session_id}/stream` - WebSocket streaming endpoint
    *   `/loupe/session/{session_id}/state` - Gets session state
    *   `/loupe/session/load` - Loads session state

**Note**: All previous route folders (auth, clone, creatives, collections, etc.) have been deleted. The backend now primarily serves as an AgentLoop wrapper with Reels-specific tools.

### 2. Reels Agent Tools (`src/agent_tools/`)

The core Reels functionality is exposed as agent tools that can be called by the AgentLoop framework during research sessions.

*   **Tool Parameters (`src/agent_tools/reels_tool_parameters.rs`)**: Defines the `ReelsToolParameters` enum with:
    *   `GenerateReel` - Generates short video reels from product/service URLs or text descriptions
    *   `BrowseWithQuery` - Fetches and processes web page content based on queries (used internally by `GenerateReel`)

*   **Tool Handlers (`src/agent_tools/handlers/`)**:
    *   `handle_generate_reel.rs` - Orchestrates reel generation by:
        1. Fetching product information from URLs (if provided) using `browse_with_query`
        2. Enhancing prompts with product details
        3. Calling the video-to-montage cloud function
        4. Storing generated reels in GCS and returning URLs
    *   `handle_reels_browse_with_query.rs` - Extracts and processes web content using Zyte for HTML extraction and screenshot services

*   **Tool Dispatch (`src/agent_tools/dispatch_reels_agent_tool.rs`)**: Routes tool calls to appropriate handlers based on the tool parameter type.

*   **Gemini Tool Conversion (`src/agent_tools/gemini_tool_conversion/`)**: Converts Reels tool schemas to Gemini's tool format for LLM integration.

### 3. External Service Integrations (`src/services/`)

*   **Google Cloud Storage (`src/services/gcs/`)**:
    *   `gcs_client.rs` - Main GCS client wrapper
    *   `gcs_service.rs` - High-level service interface
    *   `gcs_operations.rs` - GCS operations trait and implementations
    *   `production_gcs_service.rs` - Production GCS service implementation
    *   `mock_gcs_service.rs` - Mock service for testing
    *   `parse_gcs_url.rs` - URL parsing utilities
    *   `publish_website.rs` - Website publishing functionality
    *   Handles uploading, downloading, and managing objects in GCS buckets (reels, logs, assets).

*   **Screenshot Service (`src/services/screenshot/`)**:
    *   `screenshot_service.rs` - Trait definition for screenshot services
    *   `zyte_screenshot_service.rs` - Zyte-based screenshot implementation
    *   `service_factory.rs` - Factory for creating appropriate screenshot service based on configuration
    *   `mock_screenshot_service.rs` - Mock service for testing
    *   Provides screenshot capture capabilities for web content analysis.

*   **Agent Service (`src/services/agent_service.rs`)**: 
    *   Orchestrates AgentLoop state creation and configuration
    *   Manages integration of Reels tools into AgentLoop's tool schema
    *   Provides `run_and_log_research` for executing research tasks and logging results to GCS

*   **HTTP Request Service (`src/services/http_request.rs`)**: Provides HTTP client functionality for calling external APIs (e.g., video-to-montage cloud function).

### 4. External Client Libraries

*   **Zyte Client (`src/zyte/`)**: Client for interacting with Zyte API for web data extraction (HTML, screenshots, inline styles). Includes JavaScript snippets for style extraction.
*   **Webflow Client (`src/webflow/`)**: Client for interacting with the Webflow CMS API.
*   **GCP Authentication (`src/gcp_auth.rs`)**: Handles authentication with GCP services.

### 5. Utility Functions (`src/utils/`)

*   **HTML Processing**:
    *   `minimize_large_html_content.rs` - Minimizes HTML content
    *   `html_minimizer/` - Various HTML minimization utilities (SVG processing, style removal, iframe removal, etc.)
    *   `sanitize_llm_html_output.rs` - Sanitizes LLM-generated HTML
*   **Color Conversions (`src/utils/color_conversions/`)**: Utilities for converting between color formats (HSL, RGB, named colors to hex).
*   **JWT Utilities (`src/utils/jwt.rs`)**: JWT token handling utilities.
*   **Password Validation (`src/utils/password_validator.rs`)**: Password validation logic.

### 6. Middleware (`src/middleware/`)

*   **Rate Limiting (`src/middleware/rate_limit.rs`)**: Rate limiting middleware for API protection.

### 7. Error Handling (`src/errors/`)

*   `error_response.rs` - Standardized error response types
*   `permission_errors.rs` - Permission-related error types

### 8. Configuration & Setup (`src/main.rs`, `src/openapi.rs`)

*   **Main Entry Point (`src/main.rs`)**:
    *   Sets up Actix-web server with CORS (development vs production configuration)
    *   Initializes GCS client, screenshot service, and AgentLoop state
    *   Configures logging based on `APP_ENV` environment variable
    *   Sets up Swagger UI for OpenAPI documentation
    *   Registers routes via `routes::config`

*   **OpenAPI Documentation (`src/openapi.rs`)**:
    *   Uses `utoipa` for automatic OpenAPI spec generation
    *   Documents health endpoint and all AgentLoop endpoints
    *   Includes schemas for AgentLoop types (research requests, session status, messages, etc.)

### 9. Query Parser (`src/query_parser/`)

Query parsing utilities for processing search and filtering queries.

### 10. LLM Support (`src/llm_support/`)

*   `json_schema_container.rs` - Utilities for JSON schema handling in LLM interactions

## Architectural Notes

*   **Simplified Architecture**: The backend has been refactored to focus on AgentLoop integration. Most traditional database-backed routes have been removed. Database operations are handled by the AgentLoop framework itself.

*   **AgentLoop Integration**: The backend acts as a wrapper that:
    *   Initializes AgentLoop's `AppState` with Reels-specific tools
    *   Exposes AgentLoop's research and session management endpoints
    *   Provides custom Reels tools (`generate_reel`, `browse_with_query`) that AgentLoop can invoke during research sessions

*   **Tool-Based Design**: Reels functionality (reel generation, web browsing) is exposed as tools that AgentLoop agents can call, rather than direct HTTP endpoints. This allows for more flexible, AI-driven workflows.

*   **Service-Oriented**: External integrations (GCS, screenshots, HTTP) are abstracted behind service traits for testability and flexibility.

*   **One-File-Per-Item Pattern**: Adheres to strict "one item per file" guidelines (`rust_coding_guidelines.md`), using fully qualified paths instead of `use` statements in many places.

*   **No Direct Database Access**: The backend does not maintain its own database layer. Database operations are delegated to AgentLoop, which manages session state, conversation history, and research tasks.

*   **Environment Configuration**: Uses `dotenvy` for environment variable management. Key variables include:
    *   `VIDEO_TO_MONTAGE_FUNCTION_URL` - URL for video montage generation service
    *   `GCS_BUCKET_MICROSITES` - GCS bucket for storing generated reels
    *   `APP_ENV` - Environment mode (development/production)
    *   `PORT` - Server port (defaults to 8080)

*   **CORS Configuration**: Supports permissive CORS in development and configurable CORS in production to support webhook integrations.
