//! Gets per-user credit usage breakdown for an organization.
//!
//! This query returns credit usage statistics broken down by user within
//! a specific organization. It can optionally filter to specific users.
//! Used to show which users in an organization consumed credits.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserCreditUsageSummary {
    #[schema(format = "uuid")]
    pub user_id: uuid::Uuid,
    #[schema(example = "user@example.com")]
    pub user_email: String,
    #[schema(example = "450")]
    pub total_credits_used: i64,
    #[schema(example = "150")]
    pub action_count: i64,
}

/// Gets per-user credit usage breakdown for an organization.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `organization_id` - Organization to get breakdown for
/// * `start_date` - Start date in YYYY-MM-DD format
/// * `end_date` - End date in YYYY-MM-DD format
/// * `user_ids` - Optional list of specific user IDs to filter to
///
/// # Returns
///
/// Vector of `UserCreditUsageSummary` ordered by total credits used (descending)
pub async fn get_organization_user_breakdown(
    pool: &sqlx::PgPool,
    organization_id: uuid::Uuid,
    start_date: &str,
    end_date: &str,
    user_ids: Option<Vec<uuid::Uuid>>,
) -> Result<Vec<UserCreditUsageSummary>, sqlx::Error> {
    // Parse dates
    let start_naive = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    let end_naive = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let results = if let Some(ids) = user_ids {
        // Filter to specific users
        sqlx::query_as!(
            UserCreditUsageSummary,
            r#"
            SELECT 
                ct.user_id,
                u.email as user_email,
                COALESCE(SUM(ABS(ct.credits_changed)), 0)::BIGINT as "total_credits_used!",
                COUNT(*)::BIGINT as "action_count!"
            FROM credit_transactions ct
            JOIN users u ON ct.user_id = u.id
            WHERE ct.organization_id = $1
              AND DATE(ct.created_at) >= $2
              AND DATE(ct.created_at) <= $3
              AND ct.credits_changed < 0
              AND ct.user_id = ANY($4)
            GROUP BY ct.user_id, u.email
            ORDER BY 3 DESC
            "#,
            organization_id,
            start_naive,
            end_naive,
            &ids
        )
        .fetch_all(pool)
        .await?
    } else {
        // All users in the organization
        sqlx::query_as!(
            UserCreditUsageSummary,
            r#"
            SELECT 
                ct.user_id,
                u.email as user_email,
                COALESCE(SUM(ABS(ct.credits_changed)), 0)::BIGINT as "total_credits_used!",
                COUNT(*)::BIGINT as "action_count!"
            FROM credit_transactions ct
            JOIN users u ON ct.user_id = u.id
            WHERE ct.organization_id = $1
              AND DATE(ct.created_at) >= $2
              AND DATE(ct.created_at) <= $3
              AND ct.credits_changed < 0
            GROUP BY ct.user_id, u.email
            ORDER BY 3 DESC
            "#,
            organization_id,
            start_naive,
            end_naive,
        )
        .fetch_all(pool)
        .await?
    };

    log::info!(
        "Retrieved {} user credit summaries for organization {}",
        results.len(),
        organization_id
    );

    Ok(results)
}

