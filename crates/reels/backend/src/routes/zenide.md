dont use fully qualified names in utoipa annotations

use

CreateAssetRequest

isntead of

crate::a::b::c::CreateAssetRequest

When creating new structs for requests please add ToSchema derive and add schema annotations
for non standard types like this


---

## Comprehensive Guide to Route Structure and Utoipa Annotations

This guide provides a comprehensive overview of how to structure REST API routes, configure them in Actix-web, and document them using Utoipa for OpenAPI specification generation. The `crates/narrativ/backend/src/routes/creatives/` module serves as a primary example.

### 1. Module and File Organization

Routes are organized into modules based on their logical grouping (e.g., `creatives`, `styles`, `users`).

*   **Main Route Module (`crates/narrativ/backend/src/routes/`):**
    *   `mod.rs`: Declares all sub-modules (e.g., `pub mod creatives;`) and contains the main `config` function that sets up scopes for these sub-modules.
*   **Feature-Specific Route Module (e.g., `crates/narrativ/backend/src/routes/creatives/`):**
    *   `mod.rs`: Declares all handler files within this feature module (e.g., `pub mod list_creatives;`, `pub mod create_creative;`).
    *   `configure_creatives_routes.rs` (or similar name like `configure_xxx_routes.rs`): Contains a function to register all handlers for this feature.
    *   Individual `.rs` files for each API endpoint handler (e.g., `list_creatives.rs`, `create_creative.rs`).
    *   Individual `.rs` files for request and response structs specific to these handlers, if not already defined in `db` models or common DTO locations. This aligns with the "One Logical Item Per File" guideline from `rust_guidelines.md`.

### 2. Route Configuration

**a. Feature-Level Configuration (e.g., `configure_creatives_routes.rs`)**

Each feature module has a configuration function that groups its services.

```rust
// Example from: crates/narrativ/backend/src/routes/creatives/configure_creatives_routes.rs

//! Configures all Creative-related routes.
//!
//! Mounted under /api/creatives with JWT authentication.

use actix_web::web;
use super::{delete_creative, get_creative_by_id, list_creatives, /* ... other handlers ... */};

/// Sets up endpoints for Creative operations within the /api/creatives scope.
pub fn configure_creatives_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_creatives::list_creatives)
       .service(get_creative_by_id::get_creative_by_id)
       .service(create_creative::create_creative) // Assuming create_creative is imported
       .service(delete_creative::delete_creative)
       // ... other services ...
}
```

**b. Application-Level Configuration (e.g., `crates/narrativ/backend/src/routes/mod.rs`)**

The main `config` function in `routes/mod.rs` sets up scopes and applies middleware.

```rust
// Example from: crates/narrativ/backend/src/routes/mod.rs

use crate::middleware::auth::JwtMiddleware;
use crate::routes::creatives::configure_creatives_routes::configure_creatives_routes;
// ... other imports ...
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    // ... other scopes ...
    cfg.service(
        web::scope("/api") // Base scope for API v1
            .service(
                web::scope("/creatives") // Feature-specific scope
                    .wrap(JwtMiddleware) // Apply middleware like authentication
                    .configure(configure_creatives_routes), // Delegate to feature-specific config
            )
            // ... other feature scopes ...
    );
    // ... other top-level scopes ...
}
```

### 3. Handler File Structure

Each handler file (e.g., `list_creatives.rs`) should generally follow this structure:

1.  **File-level documentation (`//!`):** Briefly describe the handler's purpose.
2.  **Imports:** Necessary Actix-web types, DB models, request/response structs, error types, auth claims.
3.  **`#[utoipa::path(...)]` macro:** OpenAPI documentation for the endpoint.
4.  **Actix-web HTTP method macro:** (e.g., `#[actix_web::get(...)]`, `#[actix_web::post(...)]`).
5.  **`#[tracing::instrument(...)]` macro:** For structured logging.
6.  **Handler function:** `async fn handler_name(...) -> impl actix_web::Responder { ... }`.
7.  **`#[cfg(test)] mod tests { ... }`:** For in-file unit tests (often placeholders for API handlers requiring mocks).

### 4. `#[utoipa::path(...)]` Annotation Details

This macro is crucial for generating OpenAPI documentation.

**Example (GET with Query Parameters - from `list_creatives.rs`):**
```rust
#[utoipa::path(
    get, // HTTP Method
    path = "/api/creatives", // Full path in the API
    responses( // Define possible responses
        (status = 200, description = "List creatives with style and document names", body = ListCreativesResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    params( // Define query parameters
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("collection_id" = Option<String>, Query, description = "Filter by collection (campaign) ID"),
        // ... other query params ...
    ),
    security( // Define security requirements
        ("jwt_token" = []) // or ("bearer_auth" = []) depending on scheme name in openapi.rs
    ),
    tag = "Creatives", // Group in OpenAPI UI
)]
#[actix_web::get("")] // Path relative to the scope defined in configure_creatives_routes
#[instrument(skip(pool, auth, params))]
pub async fn list_creatives(
    pool: web::Data<PgPool>,
    auth: Claims,
    params: web::Query<ListCreativesParams>, // Struct defining query parameters
) -> impl Responder {
    // ... handler logic ...
}
```

**Example (POST with Request Body and Path Parameter - adapted from `create_creative.rs` and `delete_creative.rs`):**
```rust
#[utoipa::path(
    post, // HTTP Method
    path = "/api/creatives", // Full path
    request_body = CreateCreativeRequest, // Struct defining the request payload
    responses(
        (status = 201, description = "Created", body = Creative), // Creative is the DB model
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("")] // Path relative to scope
#[instrument(skip(pool, payload, auth))]
pub async fn create_creative(
    pool: actix_web::web::Data<sqlx::PgPool>,
    payload: actix_web::web::Json<CreateCreativeRequest>,
    auth: Claims,
) -> impl actix_web::Responder {
    // ... handler logic ...
}

// For DELETE with path parameter:
#[utoipa::path(
    delete,
    path = "/api/creatives/{id}", // Path parameter {id}
    params(
        ("id" = Uuid, Path, description = "ID of the creative to delete") // Uuid is Rust type, Path indicates it's a path param
    ),
    responses(
        (status = 204, description = "No Content"), // No body for 204
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, id, auth))]
pub async fn delete_creative(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<sqlx::types::Uuid>, // Extract path param
    auth: Claims,
) -> impl Responder {
    // ... handler logic ...
}
```

**Key `utoipa::path` attributes:**
*   **HTTP Method:** `get`, `post`, `put`, `delete`, `patch`.
*   **`path`:** The full API path for the endpoint.
*   **`request_body`:** The name of the Rust struct used for the request payload (e.g., `CreateCreativeRequest`). **Do not use fully qualified paths here.**
*   **`responses`:** A tuple array defining possible HTTP responses: `(status = HTTP_STATUS_CODE, description = "...", body = ResponseStructName)`. For responses without a body (e.g., 204), omit `body`. **Do not use fully qualified paths for `body`.**
*   **`params`:** A tuple array for path or query parameters: `("param_name" = RustType, Kind, description = "...")`.
    *   `RustType`: The Rust type of the parameter (e.g., `Uuid`, `String`, `Option<i64>`).
    *   `Kind`: `Path` for path parameters, `Query` for query parameters.
*   **`security`:** A tuple array defining security schemes: `("scheme_name_in_openapi_rs" = [])`.
*   **`tag`:** A string to group related endpoints in the OpenAPI UI.

### 5. Request and Response Struct Annotations

Structs used as request bodies or response bodies need `utoipa::ToSchema` and `serde` derives.

**General Rules (reiterated from above):**
*   **No FQNs in annotations:** When referring to types in `#[schema(value_type = ...)]` or as `body` in `responses`, use simple names like `String`, `Vec<String>`, `Option<String>`, or the struct name itself (e.g., `Creative`).
*   **`ToSchema` derive:** Add `#[derive(utoipa::ToSchema)]` to your request/response structs.
*   **`#[schema(...)]` field annotations:** Use these to provide examples and clarify types for OpenAPI, especially for `Uuid`, `Option`, `Vec`, and other non-primitive types.

**Example Request Struct (from `create_creative_request.rs`):**
```rust
//! Defines the request body structure for creating a new creative.
//! ...

/// Request payload for creating a new creative.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCreativeRequest {
    /// Optional ID of the collection this creative belongs to.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440005", format = "uuid", value_type = Option<String>)]
    pub collection_id: Option<uuid::Uuid>,
    /// Required ID of the creative format to be used.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub creative_format_id: uuid::Uuid,
    /// Optional list of document item IDs associated with this creative.
    #[schema(example = json!(["doc-uuid-1", "doc-uuid-2"]), value_type = Option<Vec<String>>)]
    pub document_ids: Option<std::vec::Vec<uuid::Uuid>>,
    // ... other fields ...
    /// URL of the HTML file in GCS for this creative.
    #[schema(example = "https://storage.googleapis.com/bucket/creative.html", value_type = String, format = "uri")]
    pub html_url: std::string::String,
}
```

**Example Response Struct (from `list_creatives.rs`):**
```rust
//! Handler for listing all creatives.
//! ...

#[derive(serde::Serialize, Debug, utoipa::ToSchema, sqlx::FromRow)]
pub struct CreativeListItem { // This struct is part of ListCreativesResponse
    #[schema(value_type = String, format = "uuid")]
    pub id: uuid::Uuid,
    #[schema(value_type = Option<String>, format = "uuid", nullable = true)]
    pub collection_id: Option<uuid::Uuid>,
    // ... other fields from Creative DB model ...
    pub html_url: Option<String>, // String types often don't need explicit value_type if it's just String
    pub is_published: bool, // bool types are also straightforward
    #[schema(value_type = String, format = "date-time")] // Explicit for DateTime
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schema(nullable = true)] // For Option<String> where example isn't critical
    pub style_name: Option<String>,
    #[schema(value_type = Option<Vec<String>>, nullable = true)] // For Option<Vec<String>>
    pub document_names: Option<Vec<String>>,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct ListCreativesResponse {
    pub items: Vec<CreativeListItem>, // Utoipa infers this is an array of CreativeListItem
    pub total_count: i64,
}
```

### 6. Updating `openapi.rs`

After creating new handlers or request/response structs:

1.  **Add Handler Paths:**
    *   In `crates/narrativ/backend/src/openapi.rs`, add the fully qualified path to your new handler function inside the `#[openapi(paths(...))]` list.
    *   Example: `crate::routes::creatives::create_creative::create_creative,`
2.  **Add Schemas:**
    *   Add any new request or response structs (or DB models used as such) to the `#[openapi(components(schemas(...)))]` list.
    *   Example: `crate::routes::creatives::create_creative_request::CreateCreativeRequest,`
    *   Example: `crate::routes::creatives::list_creatives::ListCreativesResponse,` (and `CreativeListItem` if it's not nested or picked up automatically)
    *   Example: `crate::db::creatives::Creative,` (if used directly as a response body)
3.  **Add Tags (if new):**
    *   If you introduced a new tag in `#[utoipa::path(tag = "NewTag")]`, add it to the `#[openapi(tags(...))]` list.
    *   Example: `(name = "NewTag", description = "Endpoints for managing NewThings")`

By following these guidelines, your API routes will be well-structured, consistently documented, and easily discoverable through the generated OpenAPI specification.

---

## Old Content (for reference, to be reviewed and integrated or removed if redundant with above)

(The original content of zenide.md about not using FQNs in annotations and updating openapi.rs is largely covered and expanded upon in the new section above. The example for `GenerateCreativeRequest` is also a good illustration of `#[schema(...)]` annotations.)
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct GenerateCreativeRequest {
#[schema(example = "550e8400-e29b-41d4-a716-446655440005", format = "uuid", value_type=String)]
pub collection_id: uuid::Uuid,
#[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
pub style_id: uuid::Uuid,
#[schema(example = json!(["550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]), value_type=Vec<String>)]
pub asset_ids: std::vec::Vec<uuid::Uuid>,
#[schema(example = "550e8400-e29b-41d4-a716-446655440003", format = "uuid", value_type=Option<String>)]
pub research_id: Option<uuid::Uuid>,
#[schema(example = "550e8400-e29b-41d4-a716-446655440004", format = "uuid", value_type=Option<String>)]
pub creative_format_id: Option<uuid::Uuid>,
}


also update openapi.rs module with the new route if something was created