# Backend Middleware

This directory contains custom Actix-web middleware used for processing incoming HTTP requests before they reach the main route handlers or after they have been processed.

Middleware provides a powerful mechanism to implement cross-cutting concerns like authentication, logging, request/response modification, and error handling in a modular way.

## Creating Middleware

Actix-web middleware typically involves two main components:

1.  **Middleware Factory:** A struct that implements the `actix_web::dev::Transform` trait. Its primary role is to create an instance of the actual middleware service for each worker thread.
2.  **Middleware Service:** A struct that implements the `actix_web::dev::Service` trait. This service wraps the next service in the chain (either another middleware or the final route handler) and contains the core logic executed before and/or after the wrapped service.

### Basic Template

Here's a basic structure for creating middleware:

```rust
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, Payload};
use actix_web::{Error, HttpMessage, HttpResponse, body::BoxBody};
use futures_util::future::{ok, ready, Ready, LocalBoxFuture};
use std::task::{Context, Poll};

// 1. Define the Middleware Factory (implements Transform)
pub struct MyMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for MyMiddlewareFactory
where
    // S: The type of the next service in the chain
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    // B: The type of the response body
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    // The actual middleware service type
    type Transform = MyMiddlewareService<S>;
    type InitError = ();
    // The future that resolves to the middleware service
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Create and return the middleware service instance
        ready(Ok(MyMiddlewareService { service }))
    }
}

// 2. Define the Middleware Service (implements Service)
pub struct MyMiddlewareService<S> {
    // The next service in the chain
    service: S,
}

impl<S, B> Service<ServiceRequest> for MyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    // The future type returned by the call method
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Check if the next service is ready to process requests
    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    // Process the request
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // --- Logic Before Handler ---         log::debug!("Middleware processing request: {}", req.path());

        // Example: Access request data (e.g., headers)
        // if let Some(api_key) = req.headers().get("X-API-KEY") { ... }

        // Example: Modify request extensions (to pass data downstream)
        // req.extensions_mut().insert(SomeData::new());

        // Example: Short-circuiting (e.g., authentication failure)
        // let is_valid_user = validate_user(&req);
        // if !is_valid_user {
        //     log::warn!("Unauthorized access attempt for path: {}", req.path());
        //     // Create an immediate response and stop processing
        //     let response = HttpResponse::Unauthorized().finish().map_into_boxed_body();
        //     // Use req.into_response to convert the request and response
        //     return Box::pin(async move { Ok(req.into_response(response)) });
        // }

        // Clone data needed for response processing *before* calling the next service
        // let request_id = req.extensions().get::<RequestID>().cloned();

        // Call the next service in the chain (middleware or route handler)
        let fut = self.service.call(req);

        Box::pin(async move {
            // Wait for the response from the next service
            let res: ServiceResponse<B> = fut.await?;

            // --- Logic After Handler (process response) ---
            log::debug!(
                "Middleware received response with status: {}",
                res.status()
            );

            // Example: Add a response header
            // res.headers_mut().insert(
            //     header::HeaderName::from_static("x-request-processed-by"),
            //     header::HeaderValue::from_static("my-middleware"),
            // );

            // Example: Modify the response body (requires care and matching body types)
            // let (req, res) = res.into_parts();
            // let new_res = res.map_body( ... );
            // let res = ServiceResponse::new(req, new_res);

            // Forward the final response
            Ok(res)
        })
    }
}
```

## Common Use Cases & Examples

*   **Authentication (e.g., JWT):** Verify credentials (like a JWT token from the `Authorization` header). If valid, potentially extract user information (claims) and insert it into request extensions (`req.extensions_mut().insert(claims)`) for downstream handlers. If invalid, short-circuit the request with an `HttpResponse::Unauthorized`. See `auth.rs` for a detailed JWT example.
*   **Logging:** Log request details (path, method, headers) before the handler and log the response status code after the handler completes.
*   **Data Extraction/Modification:** Read request headers, query parameters, or parts of the body. Add derived data or context objects to request extensions (`req.extensions_mut()`) for use by handlers.
*   **Header Manipulation:** Add or modify request or response headers (e.g., adding security headers like `X-Content-Type-Options`).
*   **Error Handling:** Catch errors from downstream services and transform them into appropriate HTTP responses, or implement centralized error reporting.

## Applying Middleware

Middleware is applied to the Actix-web `App` or a specific `Scope` using the `.wrap()` method during application setup. Middleware executes in the order it is wrapped (outside-in).

```rust
// Example in main.rs or app configuration
use actix_web::{web, App, HttpServer};
// Adjust import paths based on your project structure
use crate::middleware::auth::JwtMiddleware; // Assuming JwtMiddleware implements Transform
use crate::middleware::logging::LoggingMiddlewareFactory; // Example logging middleware

async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Apply LoggingMiddlewareFactory globally to all routes
            .wrap(LoggingMiddlewareFactory)
            // Apply JwtMiddleware globally (often placed early)
            .wrap(JwtMiddleware) 
            // Define scopes and routes
            .service(
                web::scope("/api")
                    // Apply another middleware only to this scope
                    // .wrap(ApiSpecificMiddleware)
                    .route("/data", web::get().to(get_data_handler))
            )
            .route("/public", web::get().to(public_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

Choose the appropriate middleware and application strategy (global vs. scoped) based on the requirements of your routes.

## Existing Middleware

### 1. JWT Authentication (`auth.rs`)

- **Factory:** `JwtMiddleware`
- **Purpose:** Verifies JSON Web Tokens (JWTs) provided in the `Authorization: Bearer <token>` header.
- **Implementation:** Implements `Transform` and `Service`. Uses `jsonwebtoken` to decode/validate the token against `JWT_SECRET` env var.
- **Behavior:**
    - On success, inserts `Claims` into request extensions.
    - On failure (missing/invalid header/token, invalid signature, missing `JWT_SECRET`), short-circuits with `HttpResponse::Unauthorized` or `HttpResponse::InternalServerError`.

### 2. User Extractor (`user_extractor.rs`)

- **Factory:** `UserExtractor`
- **Purpose:** Extracts the user identifier (`sub` field, a `Uuid`) from a JWT token in the `Authorization` header.
- **Implementation:** Implements `Transform` and `Service`. Calls `verify_jwt` from `crate::auth::tokens`.
- **Behavior:**
    - If verification succeeds, inserts the user ID (`claims.sub`) into request extensions.
    - **Note:** Unlike `JwtMiddleware`, it currently *allows requests to proceed* even if the token is missing or invalid. Downstream handlers must explicitly check for the presence of the user ID in extensions. `JwtMiddleware` is generally preferred for enforcing authentication as it rejects unauthorized requests immediately.

### 3. Credits Guard (`credits_guard.rs`)

- **Factory:** `CreditsGuard` / Helper: `RequireCredits(credits: i32)`
- **Purpose:** Validates **user** credit availability before allowing access to credit-consuming endpoints.
- **Implementation:** Implements `Transform` and `Service`. Uses `get_user_credit_allocation_by_user_id` from `crate::db::user_credit_allocation`.
- **Behavior:**
    - Extracts user ID from `AuthenticatedUser` (JWT or API Key) in request extensions.
    - Queries database for user's credit allocation and remaining credits.
    - If user has sufficient credits (≥ required amount), allows request to proceed.
    - If insufficient credits or no credit allocation found, short-circuits with `HttpResponse::PaymentRequired` (402).
    - On database errors, returns `HttpResponse::InternalServerError` (500).
    - On missing authentication, returns `HttpResponse::Unauthorized` (401).
- **Usage (User Credits Only):**
    ```rust
    use crate::middleware::credits_guard::{CreditsGuard, RequireCredits};
    
    // Using the helper function (recommended)
    .wrap(RequireCredits(5))  // Require 5 credits
    
    // Or using the struct directly
    .wrap(CreditsGuard::new(10))  // Require 10 credits
    ```
- **⚠️ Organization Credits:** This middleware **only checks user credits**. For endpoints that support organization credit contexts (via optional `organization_id`), **do NOT use this middleware**. Instead, handle credit checking in the route handler using `credits_service::deduct_credits`. See `ORGANIZATION_CREDITS_ENDPOINT_PATTERN.md` for the pattern.
- **Error Response Example:**
    ```json
    {
        "error": "Insufficient Credits",
        "message": "You need 10 credits but only have 3 credits remaining",
        "code": "INSUFFICIENT_CREDITS",
        "required_credits": 10,
        "available_credits": 3
    }
    ```
- **Requirements:**
    - Must be used after `JwtMiddleware` to ensure user authentication.
    - Database pool must be available in application data.
    - User must have a credit allocation record in the database.

## Module Declaration (`mod.rs`)

- **Purpose:** Declares the middleware modules (e.g., `pub mod auth;`, `pub mod user_extractor;`, `pub mod credits_guard;`) to make them accessible via the `crate::middleware` namespace.
