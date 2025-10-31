//! Payment completions database operations
//!
//! This module provides database operations for tracking payment completions
//! across all payment methods (cards, Apple Pay, Google Pay, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Error, FromRow};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;
use log;

/// Payment completion record
#[derive(Debug, FromRow, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaymentCompletion {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "cs_test_session_123")]
    pub session_id: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub completed_at: DateTime<Utc>,
    
    #[schema(example = "card")]
    pub payment_method: String,
    
    #[schema(example = 1000)]
    pub amount: i32,
    
    #[schema(example = "usd")]
    pub currency: String,
    
    #[schema(example = "completed")]
    pub status: String,
    
    #[schema(example = "SUMMER2024")]
    pub promo_code_used: Option<String>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Create a new payment completion record
#[instrument(skip(pool))]
pub async fn create_payment_completion(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    payment_method: &str,
    amount: i32,
    currency: &str,
    promo_code_used: Option<&str>,
) -> Result<PaymentCompletion, Error> {
    let payment_completion = sqlx::query_as!(
        PaymentCompletion,
        r#"
        INSERT INTO payment_completions 
        (user_id, session_id, payment_method, amount, currency, promo_code_used)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (session_id) DO NOTHING
        RETURNING id, user_id, session_id, completed_at, payment_method, amount, currency, status, promo_code_used, created_at
        "#,
        user_id,
        session_id,
        payment_method,
        amount,
        currency,
        promo_code_used
    )
    .fetch_optional(pool)
    .await?;

    match payment_completion {
        Some(completion) => {
            log::info!(
                "Created payment completion: {session_id} for user: {user_id} via {payment_method}"
            );
            Ok(completion)
        }
        None => {
            // Payment completion already exists, fetch the existing one
            log::info!(
                "Payment completion already exists for session: {session_id}, fetching existing record"
            );
            get_payment_completion_by_session_id(pool, session_id)
                .await?
                .ok_or_else(|| {
                    sqlx::Error::RowNotFound
                })
        }
    }
}

/// Get payment completion by session ID
#[instrument(skip(pool))]
pub async fn get_payment_completion_by_session_id(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<PaymentCompletion>, Error> {
    sqlx::query_as!(
        PaymentCompletion,
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, currency, status, promo_code_used, created_at 
        FROM payment_completions 
        WHERE session_id = $1
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await
}

/// Get recent payment completion for a user (within specified time)
#[instrument(skip(pool))]
pub async fn get_recent_payment_completion(
    pool: &PgPool,
    user_id: Uuid,
    within_minutes: i64,
) -> Result<Option<PaymentCompletion>, Error> {
    sqlx::query_as!(
        PaymentCompletion,
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, currency, status, promo_code_used, created_at
        FROM payment_completions 
        WHERE user_id = $1 
        AND completed_at > NOW() - INTERVAL '1 minute' * $2
        ORDER BY completed_at DESC 
        LIMIT 1
        "#,
        user_id,
        within_minutes as f64
    )
    .fetch_optional(pool)
    .await
}



/// Get payment history for a user
#[instrument(skip(pool))]
pub async fn get_payment_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<PaymentCompletion>, Error> {
    let limit = limit.unwrap_or(50);
    
    sqlx::query_as!(
        PaymentCompletion,
        r#"
        SELECT id, user_id, session_id, completed_at, payment_method, amount, currency, status, promo_code_used, created_at
        FROM payment_completions 
        WHERE user_id = $1 
        ORDER BY completed_at DESC 
        LIMIT $2
        "#,
        user_id,
        limit
    )
    .fetch_all(pool)
    .await
}

/// Update payment completion status
#[instrument(skip(pool))]
pub async fn update_payment_completion_status(
    pool: &PgPool,
    session_id: &str,
    status: &str,
) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_completions 
        SET status = $1
        WHERE session_id = $2
        "#,
        status,
        session_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::RowNotFound);
    }

    log::info!("Updated payment completion status: {} -> {}", session_id, status);
    Ok(())
}

/// Delete payment completion (for testing cleanup)
#[instrument(skip(pool))]
pub async fn delete_payment_completion(
    pool: &PgPool,
    session_id: &str,
) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM payment_completions 
        WHERE session_id = $1
        "#,
        session_id
    )
    .execute(pool)
    .await?;

    log::info!("Deleted payment completion: {}", session_id);
    Ok(result.rows_affected())
}

/// Get payment analytics by user
#[instrument(skip(pool))]
pub async fn get_payment_analytics(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PaymentAnalytics, Error> {
    let analytics = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_payments,
            SUM(amount) as total_amount,
            MAX(completed_at) as last_payment_date,
            COUNT(CASE WHEN payment_method = 'apple_pay' THEN 1 END) as apple_pay_count,
            COUNT(CASE WHEN payment_method = 'google_pay' THEN 1 END) as google_pay_count,
            COUNT(CASE WHEN payment_method = 'card' THEN 1 END) as card_count
        FROM payment_completions 
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(PaymentAnalytics {
        total_payments: analytics.total_payments.unwrap_or(0) as i64,
        total_amount: analytics.total_amount.unwrap_or(0) as i64,
        last_payment_date: analytics.last_payment_date,
        apple_pay_count: analytics.apple_pay_count.unwrap_or(0) as i64,
        google_pay_count: analytics.google_pay_count.unwrap_or(0) as i64,
        card_count: analytics.card_count.unwrap_or(0) as i64,
    })
}

/// Payment analytics data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentAnalytics {
    pub total_payments: i64,
    pub total_amount: i64,
    pub last_payment_date: Option<DateTime<Utc>>,
    pub apple_pay_count: i64,
    pub google_pay_count: i64,
    pub card_count: i64,
}









#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_create_payment_completion() {
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
        
        let user_id = Uuid::new_v4();
        let session_id = "cs_test_session_123";
        
        // Create a user with unique email
        let unique_email = format!("test_payment_{}@example.com", user_id.simple());
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
            user_id,
            unique_email,
            "test_password_hash"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Create payment completion
        let payment = create_payment_completion(
            &pool,
            user_id,
            session_id,
            "card",
            1000,
            "usd",
            None,
        ).await.unwrap();
        
        assert_eq!(payment.user_id, user_id);
        assert_eq!(payment.session_id, session_id);
        assert_eq!(payment.payment_method, "card");
        assert_eq!(payment.amount, 1000);
        
        // Cleanup
        delete_payment_completion(&pool, session_id).await.unwrap();
        
        // Clean up the user
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn test_get_recent_payment_completion() {
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
        
        let user_id = Uuid::new_v4();
        let session_id = "cs_test_session_456";
        
        // Create a user with unique email
        let unique_email = format!("test_recent_{}@example.com", user_id.simple());
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
            user_id,
            unique_email,
            "test_password_hash"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Create payment completion
        create_payment_completion(
            &pool,
            user_id,
            session_id,
            "apple_pay",
            2000,
            "usd",
            None,
        ).await.unwrap();
        
        // Get recent payment
        let recent = get_recent_payment_completion(&pool, user_id, 5).await.unwrap();
        assert!(recent.is_some());
        assert_eq!(recent.unwrap().payment_method, "apple_pay");
        
        // Cleanup
        delete_payment_completion(&pool, session_id).await.unwrap();
        
        // Clean up the user
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&pool)
            .await
            .unwrap();
    }
} 