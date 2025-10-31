//! OAuth2 and authentication constants
//!
//! Centralized constants for OAuth2 configuration and authentication.
//! Contains Google OAuth2 endpoints, default URLs, and placeholder values
//! used throughout the authentication system.

/// Google OAuth2 authorization endpoint URL
/// Used to redirect users to Google for authentication
pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// Google OAuth2 token exchange endpoint URL  
/// Used to exchange authorization codes for access tokens
pub const GOOGLE_TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";

/// Default OAuth2 redirect URL for development environments
/// Used when GOOGLE_REDIRECT_URL environment variable is not set
pub const DEFAULT_GOOGLE_REDIRECT_URL: &str = "http://localhost:8080/auth/google/callback";

/// Placeholder password hash for OAuth2-only users
/// These users cannot log in via password authentication as they don't have passwords
/// This is a bcrypt-format placeholder to maintain database schema compatibility
pub const OAUTH2_PLACEHOLDER_PASSWORD_HASH: &str = "$2b$12$92IXUNpkjO0rOQ5byMi.Ye4oKoEa3Ro9llC/.og/at2.uheWG/igi"; 