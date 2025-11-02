//! Test utilities and helpers for backend testing
//!
//! This module provides common functionality needed across integration and unit tests,
//! including test user management, JWT generation, and database seeding.

// This module is made public via lib.rs for tests.
// If tests need access to a crate then declare it here
// pub use crate::auth::tokens::{Claims, create_jwt}; // auth module deleted

use uuid::Uuid;
use chrono::{Utc, Duration as ChronoDuration};
use serde::{Serialize, Deserialize};
// sqlx removed - no database interaction

/// A test user that provides clean resource management for integration tests.
/// Note: Database functionality removed - sqlx dependency removed.
/// This struct is kept for API compatibility but no longer performs database operations.
pub struct TestUser {
    pub user_id: Uuid,
    pub email: String,
    pub jwt_token: String,
    pub is_admin: bool,
}

impl TestUser {
    /// Creates a new unique test user with standard (non-admin) privileges.
    /// Note: Database functionality removed - returns a placeholder user.
    pub async fn new(_pool: ()) -> Result<Self, String> {
        Self::new_with_admin(false).await
    }

    /// Creates a new unique test user with admin privileges.
    /// Note: Database functionality removed - returns a placeholder user.
    pub async fn new_admin(_pool: ()) -> Result<Self, String> {
        Self::new_with_admin(true).await
    }

    /// Internal constructor that handles both admin and non-admin users.
    /// Note: Database functionality removed - creates placeholder user data.
    async fn new_with_admin(is_admin: bool) -> Result<Self, String> {
        let user_id = Uuid::new_v4();
        let email = if is_admin {
            format!("admin-test-{}@example.com", user_id.simple())
        } else {
            format!("test-{}@example.com", user_id.simple())
        };

        // Database functionality removed - sqlx dependency removed
        // Generate placeholder JWT token
        let jwt_token = generate_test_jwt(user_id, is_admin);

        Ok(TestUser {
            user_id,
            email,
            jwt_token,
            is_admin,
        })
    }

    /// Returns the Authorization header value for HTTP requests.
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.jwt_token)
    }

    /// Clean up the test user's data from the database.
    /// Note: Database functionality removed - no-op implementation.
    pub async fn cleanup(&self) -> Result<(), String> {
        // Database functionality removed - sqlx dependency removed
        log::debug!("Cleanup called for test user {} (no-op - database removed)", self.user_id);
        Ok(())
    }
}

impl Drop for TestUser {
    /// Log when TestUser is dropped.
    fn drop(&mut self) {
        log::debug!("TestUser {} dropped - ensure cleanup() was called explicitly", self.user_id);
    }
}

/// Test-only Claims structure for JWT tokens
#[derive(Debug, Serialize, Deserialize, Default)]
struct TestClaims {
    user_id: Uuid,
    is_admin: bool,
    email: String,
    exp: u64,
}

/// Generates a test JWT token for the given user ID and admin status.
/// The token is valid for 1 hour and uses the JWT_SECRET environment variable.
pub fn generate_test_jwt(user_id: Uuid, is_admin: bool) -> String {
    use crate::utils::jwt::generate_jwt_token;
    
    let now = Utc::now();
    let exp = (now + ChronoDuration::hours(1)).timestamp();

    let claims = TestClaims {
        user_id,
        is_admin,
        email: "test@example.com".to_string(),
        exp: exp as u64,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    generate_jwt_token(&secret, "1h", &claims).expect("Failed to create test JWT")
}

/// Cleanup organization and its members from the database.
/// Note: Database functionality removed - no-op implementation.
pub async fn cleanup_org_and_members(_pool: &(), _org_id: uuid::Uuid) -> Result<(), String> {
    // Database functionality removed - sqlx dependency removed
    Ok(())
}

/// Cleanup audit logs for a specific entity from the database.
/// Note: Database functionality removed - no-op implementation.
pub async fn cleanup_audit_logs_for_entity(_pool: &(), _entity_id: uuid::Uuid) -> Result<(), String> {
    // Database functionality removed - sqlx dependency removed
    Ok(())
}

/// Cleanup organization, its members, and associated audit logs.
/// Note: Database functionality removed - no-op implementation.
pub async fn cleanup_org_and_audit_logs(_pool: &(), _org_id: uuid::Uuid) -> Result<(), String> {
    // Database functionality removed - sqlx dependency removed
    Ok(())
}

