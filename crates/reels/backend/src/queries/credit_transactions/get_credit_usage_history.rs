//! Query to get aggregated credit usage history for a user
//!
//! This query fetches credit transaction data and aggregates it by date
//! to show daily credit consumption patterns. Supports optional organization
//! filtering to show credit usage for a specific organization context.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool};
use std::str::FromStr;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

/// Aggregated credit usage for a specific date
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreditUsagePoint {
    /// Date in ISO format (YYYY-MM-DD)
    #[schema(example = "2024-01-15")]
    pub date: String,
    
    /// Total credits consumed on this date
    #[schema(example = "15")]
    pub credits_used: i64,
}

/// Get aggregated daily credit usage for a user within a date range
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID for authorization check
/// * `start_date` - Start date (inclusive) in format YYYY-MM-DD
/// * `end_date` - End date (inclusive) in format YYYY-MM-DD
/// * `organization_id` - Optional organization filter. If provided, shows that organization's usage
/// * `user_ids` - Optional list of specific user IDs to filter to (only used with organization_id)
///
/// # Returns
/// Vector of credit usage points, one per day with actual consumption.
/// Days without any transactions will not be included.
///
/// # Filtering Behavior
/// - If organization_id is Some(id) AND user_ids is Some(ids): Returns transactions for those specific users in that org
/// - If organization_id is Some(id) AND user_ids is None: Returns all transactions for that organization
/// - If organization_id is None: Returns only transactions for the authenticated user (backward compatible)
#[instrument(skip(pool))]
pub async fn get_credit_usage_history(
    pool: &PgPool,
    user_id: Uuid,
    start_date: &str,
    end_date: &str,
    organization_id: Option<Uuid>,
    user_ids: Option<Vec<Uuid>>,
) -> Result<Vec<CreditUsagePoint>, Error> {
    // Parse string dates to NaiveDate for sqlx::query! macro
    let start_naive = NaiveDate::from_str(start_date)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let end_naive = NaiveDate::from_str(end_date)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    
    let usage_points = if organization_id.is_some() && user_ids.is_some() {
        // Organization context with specific user filter
        let results = sqlx::query!(
            r#"
            SELECT 
                DATE(created_at) as date,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "credits_used!"
            FROM credit_transactions
            WHERE organization_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
              AND user_id = ANY($4)
            GROUP BY DATE(created_at)
            ORDER BY date ASC
            "#,
            organization_id,
            start_naive,
            end_naive,
            user_ids.as_ref().unwrap() as &Vec<Uuid>
        )
        .fetch_all(pool)
        .await?;
        
        results
            .into_iter()
            .map(|row| CreditUsagePoint {
                date: row.date.expect("date should not be null").format("%Y-%m-%d").to_string(),
                credits_used: row.credits_used,
            })
            .collect()
    } else if organization_id.is_some() {
        // Organization context, all users
        let results = sqlx::query!(
            r#"
            SELECT 
                DATE(created_at) as date,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "credits_used!"
            FROM credit_transactions
            WHERE organization_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
            GROUP BY DATE(created_at)
            ORDER BY date ASC
            "#,
            organization_id,
            start_naive,
            end_naive,
        )
        .fetch_all(pool)
        .await?;
        
        results
            .into_iter()
            .map(|row| CreditUsagePoint {
                date: row.date.expect("date should not be null").format("%Y-%m-%d").to_string(),
                credits_used: row.credits_used,
            })
            .collect()
    } else {
        // Personal context (no organization) - only authenticated user
        let results = sqlx::query!(
            r#"
            SELECT 
                DATE(created_at) as date,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "credits_used!"
            FROM credit_transactions
            WHERE user_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
              AND organization_id IS NULL
            GROUP BY DATE(created_at)
            ORDER BY date ASC
            "#,
            user_id,
            start_naive,
            end_naive,
        )
        .fetch_all(pool)
        .await?;
        
        results
            .into_iter()
            .map(|row| CreditUsagePoint {
                date: row.date.expect("date should not be null").format("%Y-%m-%d").to_string(),
                credits_used: row.credits_used,
            })
            .collect()
    };

    Ok(usage_points)
}

