# Backend Authentication Module (`backend/src/auth/`)

This module provides core functionalities for handling user authentication, including token generation, verification, and management.

## Guiding Principles

- **Security First:** Employ strong cryptographic practices for tokens and passwords.
- **Statelessness:** Utilize JSON Web Tokens (JWTs) for stateless session management where appropriate.
- **Clarity:** Provide clear functions for distinct authentication tasks.

## Core Components

### 1. Token Utilities (`tokens.rs`)

This submodule contains functions for generating and validating various types of tokens.

#### Verification & Password Reset Tokens

These are typically short-lived, single-use tokens used for email verification or password reset flows. They are generated as secure random strings.

```rust
// Example usage from tokens.rs
use crate::auth::tokens::{generate_verification_token, generate_password_reset_token};
use chrono::Utc;

// Generate a verification token (valid for 24 hours)
let (verification_token, verification_expires_at) = generate_verification_token();
println!("Verification Token: {}, Expires: {}", verification_token, verification_expires_at);
// Store this token and expiry associated with the user, send it via email.

// Generate a password reset token (valid for 1 hour)
let (reset_token, reset_expires_at) = generate_password_reset_token();
println!("Password Reset Token: {}, Expires: {}", reset_token, reset_expires_at);
// Store this token and expiry, send it to the user.
```

#### JSON Web Tokens (JWTs)

JWTs are used for authenticating API requests after a user has logged in. They contain claims about the user (like `user_id`) and an expiration time.

**Claims Structure:**

The `Claims` struct defines the data encoded within the JWT.

```rust
// Defined in tokens.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Claims {
    pub user_id: Uuid, // Subject (user ID)
    pub exp: u64,      // Expiration time (Unix timestamp in seconds)
    // Add other relevant claims like roles, issuer (iss), audience (aud) if needed
}
```

**JWT Generation:**

Use `create_jwt` to generate a signed JWT. Requires the `JWT_SECRET` environment variable to be set.

```rust
// Example usage from tokens.rs
use crate::auth::tokens::{create_jwt, Claims};
use chrono::{Duration, Utc};
use std::env;
use uuid::Uuid;

// Ensure JWT_SECRET is set in your environment
// env::set_var("JWT_SECRET", "your-secure-secret-key"); // Load securely in production!

let user_id = Uuid::new_v4();
let expiration = Utc::now() + Duration::days(1); // Example: 1 day validity

let claims = Claims {
    user_id,
    exp: expiration.timestamp() as u64,
};

// Assuming JWT_SECRET is loaded from env
match create_jwt(&claims) {
    Ok(token) => {
        println!("Generated JWT: {}", token);
        // Return this token to the client upon successful login
    }
    Err(e) => {
        eprintln!("Error creating JWT: {}", e);
    }
}
```

**JWT Verification:**

Use `verify_jwt` to validate an incoming JWT string and decode its claims. Also requires `JWT_SECRET`.

```rust
// Example usage from tokens.rs
use crate::auth::tokens::verify_jwt;
use std::env;

// Ensure JWT_SECRET is set (must match the one used for signing)
// env::set_var("JWT_SECRET", "your-secure-secret-key"); // Load securely in production!

let token_from_client = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."; // Replace with actual token

// Assuming JWT_SECRET is loaded from env
match verify_jwt(&token_from_client) {
    Ok(decoded_claims) => {
        println!("Token is valid! User ID: {}", decoded_claims.user_id);
        // Proceed with the request using decoded_claims.user_id
    }
    Err(e) => {
        eprintln!("Invalid token: {}", e);
        // Reject the request (e.g., return HTTP 401 Unauthorized)
    }
}
```

### 2. Password Hashing

**IMPORTANT:** This module currently *does not* provide password hashing functions. You **must** implement secure password hashing separately before storing user passwords.

- **Use a strong, adaptive hashing algorithm:** Argon2 (recommended), bcrypt, or scrypt.
- **Use a dedicated crate:** Such as `argon2` or `bcrypt`.
- **Store the full hash output:** This typically includes the algorithm identifier, salt, cost factors, and the hash itself, often in a single string format (e.g., PHC string format).

**Conceptual Example (using `argon2` crate):**

```rust
// Add `argon2 = { version = "0.5", features = ["std"] }` and `rand = "0.8"` to Cargo.toml

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

// --- Usage ---
// let hashed = hash_password("user_password123").unwrap();
// // Store `hashed` in the database

// let is_valid = verify_password("user_password123", &stored_hash).unwrap();
```

### 3. Token Types & Lifecycles

- **Access Token (JWT):** Short-lived (e.g., 15 mins - 1 hour). Used to authenticate API requests. Generated upon login or refresh.
- **Refresh Token (Secure Random String or JWT):** Longer-lived (e.g., days, weeks). Stored securely (e.g., HTTP-only cookie or secure storage). Used to obtain new access tokens without re-login. Requires careful management (storage, rotation, revocation). *This module doesn't explicitly implement refresh tokens yet.*
- **Verification Token (Secure Random String):** Short-lived (e.g., 1 hour - 1 day). Used for one-time actions like email confirmation. Generated by `generate_verification_token`.
- **Password Reset Token (Secure Random String):** Very short-lived (e.g., 1 hour). Used for one-time password reset action. Generated by `generate_password_reset_token`.

## Integration with Actix-web

JWTs are commonly used in `Authorization: Bearer <token>` headers. You can create Actix-web middleware or extractors to handle token verification.

**Conceptual Middleware Example:**

```rust
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderValue, AUTHORIZATION},
    Error, HttpMessage,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc; // Use Rc for single-threaded context like Actix-web

use crate::auth::tokens::{verify_jwt, Claims}; // Adjust path as needed

// 1. Define a struct to hold the authenticated user claims
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

// 2. Authentication Middleware Factory
pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service: Rc::new(service) })
    }
}

// 3. The Middleware itself
pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let token = match req.headers().get(AUTHORIZATION) {
                Some(header_value) => {
                    match header_value.to_str() {
                        Ok(val) if val.starts_with("Bearer ") => Some(val[7..].to_string()),
                        _ => None,
                    }
                },
                None => None,
            };

            if let Some(token_str) = token {
                match verify_jwt(&token_str) {
                    Ok(claims) => {
                        req.extensions_mut().insert(AuthenticatedUser { claims });
                        let fut = service.call(req);
                        fut.await
                    }
                    Err(_) => {
                        Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                    }
                }
            } else {
                Err(actix_web::error::ErrorUnauthorized("Authentication required"))
            }
        })
    }
}


// 4. Accessing Authenticated User in Handlers
use actix_web::{web, get, Responder, HttpRequest};

#[get("/protected")]
async fn protected_route(req: HttpRequest) -> impl Responder {
    if let Some(auth_user) = req.extensions().get::<AuthenticatedUser>() {
        format!("Hello, user ID: {}", auth_user.claims.user_id)
    } else {
        "Authentication failed or middleware not run".to_string()
    }
}

// 5. Applying the Middleware
// In your App setup:
// use actix_web::App;
// App::new()
//     .service(
//         web::scope("/api") // Apply middleware to a scope
//             .wrap(Auth)
//             .service(protected_route)
//             // Add other protected services here
//     )
//     // Add public routes outside the scope
```

**Note:** The middleware example above is conceptual. Ensure imports and error handling are adapted to your specific project structure and error types (e.g., using `anyhow::Result` as per `CODING_GUIDELINES.md`).

## Best Practices & Security Considerations

- **HTTPS Only:** Always transmit tokens and handle logins over HTTPS.
- **Secret Management:** NEVER hardcode secrets (`JWT_SECRET`, database passwords). Use environment variables, secrets managers (like Google Secret Manager), or configuration files managed securely.
- **Token Expiry:** Use reasonably short expiry times for access tokens and implement a refresh token flow for better UX and security.
- **Rate Limiting:** Protect login and token endpoints against brute-force attacks.
- **Input Validation:** Sanitize and validate all user inputs.
- **Logging:** Log authentication attempts (success and failure), but avoid logging sensitive data like passwords or full tokens.
- **Revocation:** Implement a mechanism to revoke tokens if needed (e.g., user logs out everywhere, password change, suspected compromise). This often requires maintaining a denylist or using stateful sessions for critical actions.
