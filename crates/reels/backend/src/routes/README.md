# Backend API Routes (`backend/src/routes/`)

This directory contains the route handlers and configuration for the backend API, built using the Actix-web framework. This guide provides instructions for defining routes, handling requests, integrating with the database, applying middleware, and documenting endpoints.

## Guiding Principles (Refer to `CODING_GUIDELINES.md`)

- Follow the general project guidelines outlined in `CODING_GUIDELINES.md`.
- Use `serde` for serialization/deserialization.
- Use `utoipa` for OpenAPI documentation.

## Organization

Routes are organized into modules based on functional areas (e.g., `clone.rs`). Each module typically defines:

1.  **Request/Response Structs:** Define data structures using `serde::{Deserialize, Serialize}` and `utoipa::ToSchema`.
2.  **Route Handlers:** Asynchronous functions using Actix-web macros (`#[get]`, `#[post]`, etc.).
3.  **Configuration Function:** A function (e.g., `configure_feature_routes`) to register the module's services.

## Defining Routes and Handlers

Handlers are `async` functions that take extractors (like `web::Json`, `web::Path`, `web::Data`, `web::ReqData`) and return something implementing `Responder`.

**Example Handler (Illustrative - adapt as needed):**

```rust
// backend/src/routes/example.rs (Illustrative Example)
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::auth::tokens::Claims; // For authenticated routes
use uuid::Uuid;
use anyhow::Result;

#[derive(Deserialize, ToSchema)]
pub struct CreateExampleRequest {
    #[schema(example = "My Example")]
    pub name: String,
    pub value: i32,
}

#[derive(Serialize, ToSchema)]
pub struct ExampleResponse {
    pub id: Uuid,
    pub name: String,
    pub value: i32,
}

/// Get an example resource by ID.
#[utoipa::path(
    get,
    path = "/api/example/{id}",
    tag = "Example",
    params(
        ("id" = Uuid, Path, description = "ID of the example resource")
    ),
    responses(
        (status = 200, description = "Example resource found", body = ExampleResponse),
        (status = 404, description = "Resource not found")
    ),
)]
#[get("/{id}")]
async fn get_example(
    path: web::Path<Uuid>,
) -> impl Responder {
    let example_id = path.into_inner();

    // --- Database Interaction --- 
    log::info!("Fetching example {}", example_id);
    match fetch_example(example_id).await {
        Ok(Some(example_data)) => HttpResponse::Ok().json(example_data),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Example not found"})),
        Err(e) => {
            log::error!("Failed to fetch example: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Database error"}))
        }
    }
}

/// Create a new example resource.
#[utoipa::path(
    post,
    path = "/api/example",
    tag = "Example",
    request_body = CreateExampleRequest,
    responses(
        (status = 201, description = "Example resource created", body = ExampleResponse),
        (status = 400, description = "Invalid input"),
    ),
)]
#[post("")]
async fn create_example(
    req: web::Json<CreateExampleRequest>,
) -> impl Responder {
    // --- Request Validation (Implicit via Serde) ---
    // If JSON parsing fails, Actix returns a 400 error automatically.
    // Add explicit validation if needed for business rules:
    if req.name.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Name cannot be empty"}));
    }
    if req.value < 0 {
         return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Value must be non-negative"}));
    }

    match create_example(&req.name, req.value).await {
        Ok(new_example) => HttpResponse::Created().json(new_example),
        Err(e) => {
            log::error!("Failed to create example: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create example"}))
        }
    }
}


// --- Placeholder DB Functions (Illustrative) ---
async fn fetch_example_from_db(pool: &PgPool, id: i32, user_id: Uuid) -> Result<Option<ExampleResponse>> {
    // Simulate DB fetch - replace with actual sqlx query using pool and user_id for authorization
    // Example: sqlx::query_as!(ExampleResponse, "SELECT ... WHERE id = $1 AND user_id = $2", id, user_id).fetch_optional(pool).await
    if id == 1 {
        Ok(Some(ExampleResponse { id: 1, name: "Fetched Example".to_string(), value: 100, user_id }))
    } else {
        Ok(None)
    }
}

async fn create_example_in_db(pool: &PgPool, name: &str, value: i32, user_id: Uuid) -> Result<ExampleResponse> {
    // Simulate DB insert - replace with actual sqlx query returning the created record
    // Example: sqlx::query_as!(ExampleResponse, "INSERT INTO ... RETURNING *", ...).fetch_one(pool).await
    let new_id = 2; // Simulate getting ID back from DB
    Ok(ExampleResponse { id: new_id, name: name.to_string(), value, user_id })
}


/// Configures the routes for the example feature.
pub fn configure_example_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_example)
       .service(create_example);
}

```

## Handling Requests and Responses

-   **Path Parameters:** Use `web::Path<(Type1, Type2, ...)>`. Ensure types match the route definition (e.g., `/{id}` -> `web::Path<i32>`).
-   **Query Parameters:** Use `web::Query<StructName>` where `StructName` derives `Deserialize`. Optional parameters should use `Option<T>`. Example: `/search?term=abc&page=1` -> `web::Query<SearchQuery>`. ```rust #[derive(Deserialize)] struct SearchQuery {     term: String,     page: Option<u32>, } ```
-   **JSON Body:** Use `web::Json<StructName>` where `StructName` derives `Deserialize`. Actix automatically handles JSON parsing. Invalid JSON or type mismatches result in a `400 Bad Request` response.
-   **Database Pool:** Access the shared `PgPool` using the `web::Data<PgPool>` extractor.
-   **Authentication Claims:** For routes protected by `JwtMiddleware`, extract user info with `web::ReqData<Claims>`. If the token is invalid or missing, the middleware will reject the request before it reaches the handler (typically with a `401 Unauthorized`).
-   **Responses:** Return `HttpResponse` directly or any type that implements `Responder`. Common patterns:
    -   `HttpResponse::Ok().json(data)`: 200 OK with JSON body.
    -   `HttpResponse::Created().json(data)`: 201 Created with JSON body.
    -   `HttpResponse::NoContent().finish()`: 204 No Content (for successful deletions).
    -   `HttpResponse::BadRequest().json(error_message)`: 400 Bad Request (validation errors).
    -   `HttpResponse::NotFound().json(error_message)`: 404 Not Found.
    -   `HttpResponse::Unauthorized().json(error_message)`: 401 Unauthorized (usually handled by middleware, but can be returned explicitly).
    -   `HttpResponse::Forbidden().json(error_message)`: 403 Forbidden (user authenticated but lacks permission).
    -   `HttpResponse::InternalServerError().json(error_message)`: 500 Internal Server Error (database errors, unexpected issues).
    - Use `serde_json::json!({ "error": "message" })` for simple error bodies.

## Data Serialization/Deserialization (Serde)

- Use `#[derive(Serialize, Deserialize)]` on your request and response structs.
- Use field attributes like `#[serde(rename = "newName")]` if needed.
- For database interactions, ensure struct fields match database column names or use `#[sqlx(rename = "db_column_name")]` if using `sqlx::FromRow`.

## Database Integration (SQLx)

- Pass the `web::Data<PgPool>` extractor to your handlers.
- Define database interaction logic in the corresponding `src/db/` module (e.g., `src/db/examples.rs`).
- Handler functions should call these DB functions, passing the pool.
- Handle `anyhow::Result` (or `sqlx::Result`) returned by DB functions:
    - Log errors using `log::error!`. Use `.context("...")` from `anyhow` for better error messages.
    - Map database errors (e.g., `sqlx::Error::RowNotFound`) to appropriate HTTP responses (e.g., `HttpResponse::NotFound`).
    - Return `HttpResponse::InternalServerError` for unexpected database errors.

## Middleware

Middleware is applied using `.wrap()` on a `web::scope` or individual service, typically configured in `backend/src/routes/mod.rs`.

**Example (`mod.rs`):**

```rust
// backend/src/routes/mod.rs
use crate::middleware::auth::JwtMiddleware;
use actix_web::web;

// Import route configuration functions
pub mod auth;
// pub mod example; // Assuming example.rs exists
pub mod clone;
pub mod api_keys;
pub mod requests;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Public routes (no middleware)
    cfg.service(
        web::scope("/auth")
            .configure(auth::configure_auth_routes),
    );

    // Protected API routes
    cfg.service(
        web::scope("/api")
            .wrap(JwtMiddleware) // Apply JWT Auth to all routes under /api/*
            .service(
                web::scope("/clone") 
                    .service(clone::clone_style),
            )
            .service(
                web::scope("/keys") 
                    .configure(api_keys::configure_api_key_routes),
            )
            .service(
                web::scope("/requests")
                    .configure(requests::configure_requests_routes),
            )
            // Add other protected scopes/services here
            // .service(web::scope("/example").configure(example::configure_example_routes))
    );

    // Other scopes (e.g., health, webhooks)
    cfg.service(health::health_check); // Assuming health is at root
    cfg.service(
        web::scope("/stripe")
            .configure(stripe_webhooks::configure_stripe_routes),
    );
}
```

## OpenAPI Documentation (Utoipa)

Follow these steps to ensure your endpoints are documented correctly:

1.  **Annotate Structs:**
    - Add `#[derive(ToSchema)]` to all request and response structs.
    - Use `#[schema(...)]` attributes for details:
        - `example = ...`: Provide realistic examples.
        - `value_type = ...`: Clarify types for OpenAPI (e.g., `value_type = String` for `Uuid`, `DateTime`).
        - `format = ...`: Specify formats (e.g., `format = DateTime`, `format = Uuid`).
2.  **Annotate Handlers:**
    - Add the `#[utoipa::path(...)]` macro above each handler function.
    - Specify the HTTP method (`get`, `post`, `put`, `delete`, etc.).
    - Define the `path` string, matching the Actix route.
    - Add a relevant `tag` (e.g., "Auth", "Clone", "API Keys").
    - Document parameters using `params(...)`:
        - `("param_name" = Type, Path | Query, description = "...")`
    - Specify the `request_body = StructName` if applicable.
    - List potential `responses(...)`:
        - `(status = CODE, description = "...", body = ResponseStruct | String | Value)`
        - Include success codes (200, 201, 204) and common error codes (400, 401, 403, 404, 500).
    - Indicate security requirements using `security(...)`:
        - `("user_auth" = [])` for routes requiring JWT authentication (as defined in `openapi.rs`).
3.  **Register in `openapi.rs`:**
    - Add the full path to the handler function within the `paths(...)` list in the `#[openapi(...)]` macro in `backend/src/openapi.rs`.
    - Add all relevant request and response structs (including error structs) to the `components(schemas(...))` list.
    - Ensure the tag used in the handler annotation exists in the `tags(...)` list.

Refer to existing route files (`auth.rs`, `clone.rs`, `api_keys.rs`, `requests.rs`) and `openapi.rs` for concrete examples.

## Adding New Routes

1.  **Create Module File:** Create `backend/src/routes/new_feature.rs`.
2.  **Define Structs:** Add request/response structs with `serde` and `utoipa` derives and annotations.
3.  **Implement Handlers:** Write `async` handler functions using Actix extractors, implement business logic, call database functions (defined in `src/db/new_feature.rs`), handle results, and add `#[utoipa::path]` annotations.
4.  **Create Config Function:** Add `pub fn configure_new_feature_routes(cfg: &mut web::ServiceConfig)` in the new module. Register handler functions using `cfg.service(handler_function)` or `cfg.route("/path", web::method(handler))`. 
5.  **Update `mod.rs`:**
    - Add `pub mod new_feature;`.
    - Call the new configuration function within the appropriate `web::scope` in the main `config` function (e.g., `cfg.service(web::scope("/new-feature").configure(new_feature::configure_new_feature_routes));`).
    - Ensure the scope is placed correctly (e.g., inside `/api` if it needs protection) and apply middleware like `.wrap(JwtMiddleware)` if required.
6.  **Update `openapi.rs`:** Add the new handler function paths to `paths(...)` and any new structs to `components(schemas(...))`. Add a new tag if necessary.
7.  **Database:** Create corresponding DB logic in `src/db/new_feature.rs` and necessary migrations.

## Current Route Modules

*(Refer to `backend/src/routes/mod.rs` for the definitive list and structure)*

-   **`auth.rs`**: User authentication (register, login, password reset, logout). Scope: `/auth`.
-   **`clone.rs`**: Core style cloning endpoint. Scope: `/api/clone` (Protected).
-   **`api_keys.rs`**: API key management (create, list, delete). Scope: `/api/keys` (Protected).
-   **`requests.rs`**: User request history (list, get, delete). Scope: `/api/requests` (Protected).
-   **`health.rs`**: Health check endpoint. Path: `/health`.
-   **`stripe_webhooks.rs`**: Stripe payment webhooks. Scope: `/stripe`.
