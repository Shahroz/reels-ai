//! Lists all organizations with credit information for admin viewing.
//!
//! This query extends the standard organization listing to include credit allocation data.
//! It performs a LEFT JOIN with organization_credit_allocation to include credits_remaining,
//! returning NULL for organizations without credit allocation. Supports search, filtering,
//! sorting, and pagination. Filters out personal organizations to show only team organizations
//! in the admin interface.

use sqlx::Row;

// Import the struct from its dedicated file
use crate::queries::admin::organizations::enriched_organization_with_credits::EnrichedOrganizationWithCredits;

/// Lists all organizations with enriched data including credits
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of items per page
/// * `search` - Optional search term to filter by name or owner email
/// * `sort_by` - Field to sort by (name, owner_email, created_at)
/// * `sort_order` - Sort order (asc or desc)
///
/// # Returns
///
/// A tuple containing a vector of EnrichedOrganizationWithCredits and the total count
#[tracing::instrument(skip(pool))]
pub async fn list_organizations_with_credits(
    pool: &sqlx::PgPool,
    page: i64,
    limit: i64,
    search: Option<&str>,
    sort_by: &str,
    sort_order: &str,
) -> anyhow::Result<(Vec<EnrichedOrganizationWithCredits>, i64)> {
    let offset = (page - 1) * limit;
    
    let search_pattern = search.map(|s| format!("%{}%", s.to_lowercase()));
    
    // Build query based on sort parameters without string interpolation
    // This prevents SQL injection while maintaining sort functionality
    let query_str = match (sort_by, sort_order) {
        ("name", "asc") => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY o.name ASC
            LIMIT $2 OFFSET $3
        "#,
        ("name", "desc") => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY o.name DESC
            LIMIT $2 OFFSET $3
        "#,
        ("owner_email", "asc") => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY u.email ASC
            LIMIT $2 OFFSET $3
        "#,
        ("owner_email", "desc") => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY u.email DESC
            LIMIT $2 OFFSET $3
        "#,
        ("created_at", "asc") => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY o.created_at ASC
            LIMIT $2 OFFSET $3
        "#,
        // Default to created_at DESC for any other combination
        _ => r#"
            SELECT 
                o.id, o.name, o.owner_user_id, u.email as owner_email,
                COALESCE(COUNT(om.user_id), 0) as member_count,
                o.created_at, o.updated_at, oca.credits_remaining
            FROM organizations o
            INNER JOIN users u ON o.owner_user_id = u.id
            LEFT JOIN organization_members om ON o.id = om.organization_id
            LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
            WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
              AND o.is_personal = false
            GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at, oca.credits_remaining
            ORDER BY o.created_at DESC
            LIMIT $2 OFFSET $3
        "#,
    };
    
    let count_query_str = r#"
        SELECT COUNT(DISTINCT o.id) as count
        FROM organizations o
        INNER JOIN users u ON o.owner_user_id = u.id
        WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
          AND o.is_personal = false
    "#;
    
    let rows = sqlx::query(query_str)
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    
    let orgs: Vec<EnrichedOrganizationWithCredits> = rows
        .iter()
        .map(|row| <EnrichedOrganizationWithCredits as sqlx::FromRow<sqlx::postgres::PgRow>>::from_row(row))
        .collect::<Result<Vec<_>, _>>()?;
    
    let count_row = sqlx::query(count_query_str)
        .bind(search_pattern.as_deref())
        .fetch_one(pool)
        .await?;
    
    let total_count: i64 = count_row.try_get("count")?;
    
    Ok((orgs, total_count))
}

