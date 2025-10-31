//! Lists all users with their personal organization credits for admin viewing.
//!
//! This query extends the standard user listing to include credit information from
//! each user's personal organization. It performs a LEFT JOIN with organizations and
//! organization_credit_allocation to include credits, returning NULL for users who
//! don't have a personal organization set up yet. Supports filtering, sorting, and
//! pagination for comprehensive admin interfaces.

use sqlx::Row;

// Import the struct from its dedicated file
use crate::queries::admin::users::enriched_user::EnrichedUser;

/// Lists all users with personal organization credit information
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of items per page
/// * `sort_by` - Field to sort by (created_at, email, status)
/// * `sort_order` - Sort order (asc or desc)
/// * `search` - Optional search term to filter by email
/// * `status_filter` - Optional status filter
///
/// # Returns
///
/// A tuple containing a vector of EnrichedUser and the total count
#[tracing::instrument(skip(pool))]
pub async fn list_users_with_credits(
    pool: &sqlx::PgPool,
    page: i64,
    limit: i64,
    sort_by: &str,
    sort_order: &str,
    search: Option<&str>,
    status_filter: Option<&str>,
) -> anyhow::Result<(Vec<EnrichedUser>, i64)> {
    let offset = (page - 1) * limit;
    
    let search_pattern = search.map(|s| format!("%{}%", s.to_lowercase()));
    
    // Build query based on sort parameters without string interpolation
    // This prevents SQL injection while maintaining sort functionality
    let query_str = match (sort_by, sort_order) {
        ("email", "asc") => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.email ASC
            LIMIT $3 OFFSET $4
        "#,
        ("email", "desc") => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.email DESC
            LIMIT $3 OFFSET $4
        "#,
        ("status", "asc") => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.status ASC
            LIMIT $3 OFFSET $4
        "#,
        ("status", "desc") => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.status DESC
            LIMIT $3 OFFSET $4
        "#,
        ("created_at", "asc") => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.created_at ASC
            LIMIT $3 OFFSET $4
        "#,
        // Default to created_at DESC for any other combination
        _ => r#"
            SELECT 
                u.id, u.email, u.status, u.is_admin, u.feature_flags, u.created_at,
                oca.credits_remaining, o.id as personal_org_id
            FROM users u
            LEFT JOIN organizations o ON u.id = o.owner_user_id AND o.is_personal = true
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
              AND ($2::TEXT IS NULL OR u.status = $2)
            ORDER BY u.created_at DESC
            LIMIT $3 OFFSET $4
        "#,
    };
    
    let count_query_str = r#"
        SELECT COUNT(*) as count
        FROM users u
        WHERE 
            ($1::TEXT IS NULL OR LOWER(u.email) LIKE $1)
            AND ($2::TEXT IS NULL OR u.status = $2)
    "#;
    
    let rows = sqlx::query(query_str)
        .bind(search_pattern.as_deref())
        .bind(status_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    
    let users: Vec<EnrichedUser> = rows
        .iter()
        .map(|row| <EnrichedUser as sqlx::FromRow<sqlx::postgres::PgRow>>::from_row(row))
        .collect::<Result<Vec<_>, _>>()?;
    
    let count_row = sqlx::query(count_query_str)
        .bind(search_pattern.as_deref())
        .bind(status_filter)
        .fetch_one(pool)
        .await?;
    
    let total_count: i64 = count_row.try_get("count")?;
    
    Ok((users, total_count))
}

