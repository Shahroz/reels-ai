//! Retrieves credit usage breakdown by action type.
//!
//! Queries credit transactions grouped by action_type with optional
//! filtering by organization and user IDs.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Added revision history (lighter-weight approach: keeping use statements)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

/// Action type breakdown point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActionTypeBreakdown {
    /// Type of action (e.g., "retouch_image", "generate_creative")
    #[schema(example = "retouch_image")]
    pub action_type: String,
    
    /// Total credits consumed for this action type
    #[schema(example = "150")]
    pub total_credits_used: i64,
    
    /// Number of times this action was performed
    #[schema(example = "25")]
    pub action_count: i64,
}

/// Get action type breakdown for a user within a date range
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID for authorization check
/// * `start_date` - Start date (inclusive) in format YYYY-MM-DD
/// * `end_date` - End date (inclusive) in format YYYY-MM-DD
/// * `organization_id` - Optional organization filter. If provided, shows that organization's usage
/// * `user_ids` - Optional list of specific user IDs to filter to (only used with organization_id)
///
/// # Filtering Behavior
/// - If organization_id is Some(id) AND user_ids is Some(ids): Returns transactions for those specific users in that org
/// - If organization_id is Some(id) AND user_ids is None: Returns all transactions for that organization
/// - If organization_id is None: Returns only transactions for the authenticated user (backward compatible)
#[instrument(skip(pool))]
pub async fn get_action_type_breakdown(
    pool: &PgPool,
    user_id: Uuid,
    start_date: &str,
    end_date: &str,
    organization_id: Option<Uuid>,
    user_ids: Option<Vec<Uuid>>,
) -> Result<Vec<ActionTypeBreakdown>, sqlx::Error> {
    // Parse string dates to NaiveDate for sqlx::query! macro
    let start_naive = NaiveDate::from_str(start_date)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let end_naive = NaiveDate::from_str(end_date)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    
    let breakdown = if organization_id.is_some() && user_ids.is_some() {
        // Organization context with specific user filter
        let results = sqlx::query!(
            r#"
            SELECT 
                action_type,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "total_credits_used!",
                COUNT(*)::BIGINT as "action_count!"
            FROM credit_transactions
            WHERE organization_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
              AND user_id = ANY($4)
            GROUP BY action_type
            ORDER BY 2 DESC
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
            .map(|row| ActionTypeBreakdown {
                action_type: row.action_type,
                total_credits_used: row.total_credits_used,
                action_count: row.action_count,
            })
            .collect()
    } else if organization_id.is_some() {
        // Organization context, all users
        let results = sqlx::query!(
            r#"
            SELECT 
                action_type,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "total_credits_used!",
                COUNT(*)::BIGINT as "action_count!"
            FROM credit_transactions
            WHERE organization_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
            GROUP BY action_type
            ORDER BY 2 DESC
            "#,
            organization_id,
            start_naive,
            end_naive,
        )
        .fetch_all(pool)
        .await?;
        
        results
            .into_iter()
            .map(|row| ActionTypeBreakdown {
                action_type: row.action_type,
                total_credits_used: row.total_credits_used,
                action_count: row.action_count,
            })
            .collect()
    } else {
        // Personal context (no organization) - only authenticated user
        let results = sqlx::query!(
            r#"
            SELECT 
                action_type,
                COALESCE(SUM(ABS(credits_changed)), 0)::BIGINT as "total_credits_used!",
                COUNT(*)::BIGINT as "action_count!"
            FROM credit_transactions
            WHERE user_id = $1
              AND DATE(created_at) >= $2
              AND DATE(created_at) <= $3
              AND credits_changed < 0
              AND organization_id IS NULL
            GROUP BY action_type
            ORDER BY 2 DESC
            "#,
            user_id,
            start_naive,
            end_naive,
        )
        .fetch_all(pool)
        .await?;
        
        results
            .into_iter()
            .map(|row| ActionTypeBreakdown {
                action_type: row.action_type,
                total_credits_used: row.total_credits_used,
                action_count: row.action_count,
            })
            .collect()
    };

    Ok(breakdown)
}

