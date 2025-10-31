//! Billing database models and operations
//!
//! This module provides database models for billing-related entities
//! that map to existing tables in the database.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Error, FromRow};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

/// Webhook event record for idempotency
#[derive(Debug, FromRow, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookEvent {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "evt_test_webhook_123")]
    pub event_id: String,
    
    #[schema(example = "checkout.session.completed")]
    pub event_type: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub processed_at: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(example = "cs_test_checkout_123")]
    pub checkout_session_id: Option<String>,
    
    #[schema(example = "SUMMER2024")]
    pub promo_code: Option<String>,
}

/// Checkout session record
#[derive(Debug, FromRow, Clone, Serialize, Deserialize, ToSchema)]
pub struct CheckoutSession {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "cs_test_checkout_123")]
    pub stripe_checkout_id: String,
    
    #[schema(example = "price_test_plan_123")]
    pub plan_id: String,
    
    #[schema(example = "pending")]
    pub status: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub completed_at: Option<DateTime<Utc>>,
    
    #[schema(example = "{\"source\": \"web\"}")]
    pub metadata: Option<serde_json::Value>,
}

/// Create a new webhook event record
/// 
/// Returns (WebhookEvent, is_new) where is_new indicates if this was newly created (true) or already existed (false)
#[instrument(skip(pool))]
pub async fn create_webhook_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    checkout_session_id: Option<&str>,
    promo_code: Option<&str>,
) -> Result<(WebhookEvent, bool), Error> {
    let webhook_event = sqlx::query_as!(
        WebhookEvent,
        r#"
        INSERT INTO webhook_events 
        (event_id, event_type, processed_at, checkout_session_id, promo_code)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (event_id) DO NOTHING
        RETURNING id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        "#,
        event_id,
        event_type,
        Utc::now(),
        checkout_session_id,
        promo_code
    )
    .fetch_optional(pool)
    .await?;

    match webhook_event {
        Some(event) => Ok((event, true)), // Newly created
        None => {
            // Webhook event already exists, fetch the existing one
            let existing_event = get_webhook_event_by_id(pool, event_id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound)?;
            Ok((existing_event, false)) // Already existed
        }
    }
}

/// Get webhook event by event ID
#[instrument(skip(pool))]
pub async fn get_webhook_event_by_id(pool: &PgPool, event_id: &str) -> Result<Option<WebhookEvent>, Error> {
    sqlx::query_as!(
        WebhookEvent,
        r#"
        SELECT id, event_id, event_type, processed_at, created_at, checkout_session_id, promo_code
        FROM webhook_events
        WHERE event_id = $1
        "#,
        event_id
    )
    .fetch_optional(pool)
    .await
}

/// Check if webhook event has been processed
#[instrument(skip(pool))]
pub async fn is_webhook_event_processed(pool: &PgPool, event_id: &str) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM webhook_events
        WHERE event_id = $1
        "#,
        event_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count.unwrap_or(0) > 0)
}

/// Create a new checkout session record
#[instrument(skip(pool))]
pub async fn create_checkout_session(
    pool: &PgPool,
    user_id: Uuid,
    stripe_checkout_id: &str,
    plan_id: &str,
    metadata: Option<serde_json::Value>,
) -> Result<CheckoutSession, Error> {
    sqlx::query_as!(
        CheckoutSession,
        r#"
        INSERT INTO checkout_sessions 
        (user_id, stripe_checkout_id, plan_id, metadata)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, stripe_checkout_id, plan_id, status, created_at, updated_at, completed_at, metadata
        "#,
        user_id,
        stripe_checkout_id,
        plan_id,
        metadata
    )
    .fetch_one(pool)
    .await
}

/// Get checkout session by Stripe checkout ID
#[instrument(skip(pool))]
pub async fn get_checkout_session_by_stripe_id(pool: &PgPool, stripe_checkout_id: &str) -> Result<Option<CheckoutSession>, Error> {
    sqlx::query_as!(
        CheckoutSession,
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE stripe_checkout_id = $1
        "#,
        stripe_checkout_id
    )
    .fetch_optional(pool)
    .await
}

/// Update checkout session status
#[instrument(skip(pool))]
pub async fn update_checkout_session_status(
    pool: &PgPool,
    stripe_checkout_id: &str,
    status: &str,
    completed_at: Option<DateTime<Utc>>,
) -> Result<CheckoutSession, Error> {
    sqlx::query_as!(
        CheckoutSession,
        r#"
        UPDATE checkout_sessions 
        SET status = $1, completed_at = $2, updated_at = $3
        WHERE stripe_checkout_id = $4
        RETURNING id, user_id, stripe_checkout_id, plan_id, status, created_at, updated_at, completed_at, metadata
        "#,
        status,
        completed_at,
        Utc::now(),
        stripe_checkout_id
    )
    .fetch_one(pool)
    .await
}

/// Get checkout sessions for a user
#[instrument(skip(pool))]
pub async fn get_user_checkout_sessions(pool: &PgPool, user_id: Uuid) -> Result<Vec<CheckoutSession>, Error> {
    sqlx::query_as!(
        CheckoutSession,
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

/// Clean up old webhook events (older than 30 days)
#[instrument(skip(pool))]
pub async fn cleanup_old_webhook_events(pool: &PgPool) -> Result<u64, Error> {
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(30);
    
    let result = sqlx::query!(
        "DELETE FROM webhook_events WHERE processed_at < $1",
        cutoff_date
    )
    .execute(pool)
    .await?;
    
    log::info!("Cleaned up {} old webhook events", result.rows_affected());
    Ok(result.rows_affected())
}
