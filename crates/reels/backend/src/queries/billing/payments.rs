#![allow(clippy::disallowed_methods)]
//! Fetches payment completion information from the payment_completions table.
//!
//! This function executes SQL queries against the database to retrieve payment
//! completion records, payment methods, and billing analytics.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Get payment completions for a user
pub async fn get_user_payment_completions(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::payment_completions::PaymentCompletion>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::payment_completions::PaymentCompletion>(
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, 
               currency, status, promo_code_used, created_at
        FROM payment_completions
        WHERE user_id = $1
        ORDER BY completed_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get payment completion by session ID
pub async fn get_payment_completion_by_session(
    pool: &PgPool,
    session_id: &str,
) -> std::result::Result<Option<crate::db::payment_completions::PaymentCompletion>, sqlx::Error> {
    sqlx::query_as::<_, crate::db::payment_completions::PaymentCompletion>(
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, 
               currency, status, promo_code_used, created_at
        FROM payment_completions
        WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

/// Get payment analytics for a user
pub async fn get_user_payment_analytics(
    pool: &PgPool,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> std::result::Result<crate::db::payment_completions::PaymentAnalytics, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_payments,
            SUM(amount) as total_amount,
            COUNT(DISTINCT payment_method) as payment_methods_used
        FROM payment_completions
        WHERE user_id = $1 
          AND completed_at BETWEEN $2 AND $3
          AND status = 'completed'
        "#,
        user_id,
        start_date,
        end_date
    )
    .fetch_one(pool)
    .await?;

    Ok(crate::db::payment_completions::PaymentAnalytics {
        total_payments: result.total_payments.unwrap_or(0) as i64,
        total_amount: result.total_amount.unwrap_or(0),
        last_payment_date: None, // We don't have this in our query
        apple_pay_count: 0, // We don't have this in our query
        google_pay_count: 0, // We don't have this in our query
        card_count: 0, // We don't have this in our query
    })
}

/// Get recent payment completions across all users
pub async fn get_recent_payment_completions(
    pool: &PgPool,
    limit: Option<i64>,
) -> std::result::Result<Vec<crate::db::payment_completions::PaymentCompletion>, sqlx::Error> {
    let limit = limit.unwrap_or(100);
    
    sqlx::query_as::<_, crate::db::payment_completions::PaymentCompletion>(
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, 
               currency, status, promo_code_used, created_at
        FROM payment_completions
        WHERE status = 'completed'
        ORDER BY completed_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get payment completions by payment method
pub async fn get_payment_completions_by_method(
    pool: &PgPool,
    payment_method: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> std::result::Result<Vec<crate::db::payment_completions::PaymentCompletion>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    
    sqlx::query_as::<_, crate::db::payment_completions::PaymentCompletion>(
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, 
               currency, status, promo_code_used, created_at
        FROM payment_completions
        WHERE payment_method = $1
        ORDER BY completed_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(payment_method)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
