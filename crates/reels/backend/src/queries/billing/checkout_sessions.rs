#![allow(clippy::disallowed_methods)]
//! Fetches checkout session information from the checkout_sessions table.
//!
//! This function executes SQL queries against the database to retrieve checkout
//! session records, status tracking, and session analytics.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Checkout session analytics data
#[derive(Debug, Clone)]
pub struct CheckoutSessionAnalytics {
    pub total_sessions: i64,
    pub completed_sessions: i64,
    pub failed_sessions: i64,
    pub pending_sessions: i64,
    pub unique_users: i64,
    pub unique_plans: i64,
}

/// Get checkout sessions for a user with pagination
pub async fn get_user_checkout_sessions(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::CheckoutSession>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::billing::CheckoutSession>(
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, 
               updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get checkout sessions by status
pub async fn get_checkout_sessions_by_status(
    pool: &PgPool,
    status: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::CheckoutSession>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::billing::CheckoutSession>(
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, 
               updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE status = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get checkout sessions by plan ID
pub async fn get_checkout_sessions_by_plan(
    pool: &PgPool,
    plan_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::CheckoutSession>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::billing::CheckoutSession>(
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, 
               updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE plan_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(plan_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get checkout sessions created within a date range
pub async fn get_checkout_sessions_by_date_range(
    pool: &PgPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    limit: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::CheckoutSession>, sqlx::Error> {
    let limit = limit.unwrap_or(1000);
    
    sqlx::query_as::<_, crate::db::billing::CheckoutSession>(
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, 
               updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE created_at BETWEEN $1 AND $2
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get checkout session analytics
pub async fn get_checkout_session_analytics(
    pool: &PgPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> std::result::Result<CheckoutSessionAnalytics, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_sessions,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_sessions,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_sessions,
            COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_sessions,
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(DISTINCT plan_id) as unique_plans
        FROM checkout_sessions
        WHERE created_at BETWEEN $1 AND $2
        "#,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    Ok(CheckoutSessionAnalytics {
        total_sessions: result.total_sessions.unwrap_or(0) as i64,
        completed_sessions: result.completed_sessions.unwrap_or(0) as i64,
        failed_sessions: result.failed_sessions.unwrap_or(0) as i64,
        pending_sessions: result.pending_sessions.unwrap_or(0) as i64,
        unique_users: result.unique_users.unwrap_or(0) as i64,
        unique_plans: result.unique_plans.unwrap_or(0) as i64,
    })
}

/// Get checkout sessions that need attention (failed or expired)
pub async fn get_checkout_sessions_needing_attention(
    pool: &PgPool,
    limit: Option<i64>,
) -> std::result::Result<Vec<crate::db::billing::CheckoutSession>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    
    sqlx::query_as::<_, crate::db::billing::CheckoutSession>(
        r#"
        SELECT id, user_id, stripe_checkout_id, plan_id, status, created_at, 
               updated_at, completed_at, metadata
        FROM checkout_sessions
        WHERE status IN ('failed', 'expired')
          AND created_at >= NOW() - INTERVAL '7 days'
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Create a new checkout session
pub async fn create_checkout_session(
    pool: &PgPool,
    user_id: Uuid,
    stripe_checkout_id: &str,
    plan_id: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO checkout_sessions (user_id, stripe_checkout_id, plan_id, status)
        VALUES ($1, $2, $3, 'pending')
        "#,
        user_id,
        stripe_checkout_id,
        plan_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update checkout session status
pub async fn update_checkout_session_status(
    pool: &PgPool,
    stripe_checkout_id: &str,
    status: &str,
    metadata: Option<serde_json::Value>,
) -> std::result::Result<(), sqlx::Error> {
    let completed_at = if status == "completed" {
        Some(Utc::now())
    } else {
        None
    };
    
    sqlx::query!(
        r#"
        UPDATE checkout_sessions 
        SET status = $1, updated_at = NOW(), completed_at = $2, metadata = $3
        WHERE stripe_checkout_id = $4
        "#,
        status,
        completed_at,
        metadata,
        stripe_checkout_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
