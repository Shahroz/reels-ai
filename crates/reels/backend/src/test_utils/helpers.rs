//! Test utilities and helpers for backend testing
//!
//! This module provides common functionality needed across integration and unit tests,
//! including test user management, JWT generation, and database seeding.

// This module is made public via lib.rs for tests.
// If tests need access to a crate then declare it here
pub use crate::auth::tokens::{Claims, create_jwt};
pub use crate::db::documents::Document;
pub use crate::routes::documents::create_document_request::CreateDocumentRequest;

use uuid::Uuid;
use chrono::{Utc, Duration as ChronoDuration};
use sqlx::PgPool;

/// A test user that provides clean resource management for integration tests.
/// Call `cleanup()` explicitly at the end of tests to ensure proper resource cleanup.
pub struct TestUser {
    pub user_id: Uuid,
    pub email: String,
    pub jwt_token: String,
    pub is_admin: bool,
    pool: PgPool,
}

impl TestUser {
    /// Creates a new unique test user with standard (non-admin) privileges.
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        Self::new_with_admin(pool, false).await
    }

    /// Creates a new unique test user with admin privileges.
    pub async fn new_admin(pool: PgPool) -> Result<Self, sqlx::Error> {
        Self::new_with_admin(pool, true).await
    }

    /// Internal constructor that handles both admin and non-admin users.
    async fn new_with_admin(pool: PgPool, is_admin: bool) -> Result<Self, sqlx::Error> {
        let user_id = Uuid::new_v4();
        let email = if is_admin {
            format!("admin-test-{}@example.com", user_id.simple())
        } else {
            format!("test-{}@example.com", user_id.simple())
        };

        // Create user with a single, simple transaction
        let password_hash = "$2b$12$e0NRnKMBd0KZyLDBsjg2EekNkHEGj0ERZ3XQWhaDhjqmhL8uM0f9C";
        
        sqlx::query!(
            r#"
            INSERT INTO users (
                id, email, password_hash, stripe_customer_id, email_verified, is_admin,
                created_at, updated_at, verification_token, token_expiry, subscription_status
            ) VALUES (
                $1, $2, $3, NULL, true, $4, NOW(), NOW(), NULL, NULL, 'active'
            )
            "#,
            user_id,
            email,
            password_hash,
            is_admin
        )
        .execute(&pool)
        .await?;

        // Create credit allocation for test user (free plan with 5 daily credits, 30 plan credits)
        sqlx::query!(
            r#"
            INSERT INTO user_credit_allocation (
                user_id, plan_type, daily_credits, plan_credits, credits_remaining, 
                credit_limit, last_daily_credit_claimed_at
            ) VALUES (
                $1, 'free', 5, 30, 30, 30, NOW()
            )
            "#,
            user_id
        )
        .execute(&pool)
        .await?;

        // Create personal organization for test user
        let org_name = format!("{}'s Personal Workspace", email);
        let org_id: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO organizations (name, owner_user_id, is_personal)
            VALUES ($1, $2, true)
            RETURNING id
            "#,
            org_name,
            user_id
        )
        .fetch_one(&pool)
        .await?;

        // Add user as owner member
        sqlx::query!(
            r#"
            INSERT INTO organization_members (organization_id, user_id, role, status, joined_at)
            VALUES ($1, $2, 'owner', 'active', NOW())
            "#,
            org_id,
            user_id
        )
        .execute(&pool)
        .await?;

        // Create organization credit allocation (matching user credits)
        sqlx::query!(
            r#"
            INSERT INTO organization_credit_allocation (organization_id, credits_remaining, last_reset_date)
            VALUES ($1, 30, NOW())
            "#,
            org_id
        )
        .execute(&pool)
        .await?;

        let jwt_token = generate_test_jwt(user_id, is_admin);

        Ok(TestUser {
            user_id,
            email,
            jwt_token,
            is_admin,
            pool,
        })
    }

    /// Returns the Authorization header value for HTTP requests.
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.jwt_token)
    }

    /// Clean up the test user's data from the database.
    /// Call this at the end of tests to ensure proper resource cleanup.
    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        // Simple cleanup - just delete the user and let cascading handle dependencies
        sqlx::query!("DELETE FROM user_credit_allocation WHERE user_id = $1", self.user_id)
            .execute(&self.pool)
            .await?;

        sqlx::query!("DELETE FROM credit_transactions WHERE user_id = $1", self.user_id)
            .execute(&self.pool)
            .await?;

        sqlx::query!("DELETE FROM users WHERE id = $1", self.user_id)
            .execute(&self.pool)
            .await?;
        log::debug!("Cleanup completed for test user {}", self.user_id);
        Ok(())
    }
}

impl Drop for TestUser {
    /// Log when TestUser is dropped.
    fn drop(&mut self) {
        log::debug!("TestUser {} dropped - ensure cleanup() was called explicitly", self.user_id);
    }
}

/// Generates a test JWT token for the given user ID and admin status.
/// The token is valid for 1 hour and uses the JWT_SECRET environment variable.
pub fn generate_test_jwt(user_id: Uuid, is_admin: bool) -> String {
    let now = Utc::now();
    let exp = (now + ChronoDuration::hours(1)).timestamp();

    let claims = Claims {
        user_id,
        is_admin,
        email: "test@example.com".to_string(),
        exp: exp as u64,
        ..Default::default()
    };

    create_jwt(&claims).expect("Failed to create test JWT")
}

/// Cleanup organization and its members from the database.
/// Used in admin organization tests to ensure proper resource cleanup.
pub async fn cleanup_org_and_members(pool: &sqlx::PgPool, org_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM organization_members WHERE organization_id = $1", org_id)
        .execute(pool)
        .await?;
    
    sqlx::query!("DELETE FROM organizations WHERE id = $1", org_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Cleanup audit logs for a specific entity from the database.
/// Used in admin tests to ensure proper resource cleanup.
pub async fn cleanup_audit_logs_for_entity(pool: &sqlx::PgPool, entity_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM audit_logs WHERE target_entity_id = $1", entity_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Cleanup organization, its members, and associated audit logs.
/// Combines cleanup operations for convenience in tests.
pub async fn cleanup_org_and_audit_logs(pool: &sqlx::PgPool, org_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    cleanup_audit_logs_for_entity(pool, org_id).await?;
    cleanup_org_and_members(pool, org_id).await?;
    Ok(())
}

