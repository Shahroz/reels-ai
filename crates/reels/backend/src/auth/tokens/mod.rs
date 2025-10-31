//! JWT authentication and token management module.
//!
//! Provides comprehensive JWT token creation, verification, and claims management
//! for user authentication and authorization. Includes support for admin impersonation,
//! feature flags, and secure token generation for email verification and password reset.
//! All functions follow dependency injection patterns for testability and security.

// Core JWT functionality
pub mod claims;
pub mod get_jwt_secret;
pub mod create_jwt_with_secret;
pub mod create_jwt;
pub mod verify_jwt_with_secret;
pub mod verify_jwt;

// Token generation for workflows
pub mod generate_verification_token;
pub mod generate_password_reset_token;

// Magic link authentication
pub mod magic_link_claims;
pub mod generate_magic_link_jwt;
pub mod verify_magic_link_jwt;

// HTTP integration
pub mod claims_from_request;

// Re-export the main types and functions for convenience
pub use claims::Claims;
pub use get_jwt_secret::{get_jwt_secret, validate_jwt_secret_on_startup};
pub use create_jwt_with_secret::create_jwt_with_secret;
pub use create_jwt::create_jwt;
pub use verify_jwt_with_secret::verify_jwt_with_secret;
pub use verify_jwt::verify_jwt;
pub use generate_verification_token::generate_verification_token;
pub use generate_password_reset_token::generate_password_reset_token;
pub use magic_link_claims::MagicLinkClaims;
pub use generate_magic_link_jwt::generate_magic_link_jwt;
pub use verify_magic_link_jwt::verify_magic_link_jwt;
