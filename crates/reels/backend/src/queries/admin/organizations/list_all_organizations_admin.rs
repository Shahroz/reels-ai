//! Lists all organizations with enriched data for admin viewing.
//!
//! This query allows admins to view all organizations in the system, not just those
//! they belong to. It includes owner email, member count, and supports filtering by
//! search term (matches name or owner email) and sorting by various fields.
//! Returns paginated results with total count for pagination controls.

use sqlx::Row;

pub struct EnrichedOrganization {
    pub id: uuid::Uuid,
    pub name: String,
    pub owner_user_id: uuid::Uuid,
    pub owner_email: String,
    pub member_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_all_organizations_admin(
    pool: &sqlx::PgPool,
    page: i64,
    limit: i64,
    search: Option<&str>,
    sort_by: &str,
    sort_order: &str,
) -> anyhow::Result<(Vec<EnrichedOrganization>, i64)> {
    let offset = (page - 1) * limit;

    let search_pattern = search.map(|s| format!("%{}%", s.to_lowercase()));

    let order_clause = match (sort_by, sort_order) {
        ("name", "asc") => "o.name ASC",
        ("name", "desc") => "o.name DESC",
        ("owner_email", "asc") => "u.email ASC",
        ("owner_email", "desc") => "u.email DESC",
        ("created_at", "desc") => "o.created_at DESC",
        ("created_at", "asc") => "o.created_at ASC",
        _ => "o.created_at DESC", // Default
    };

    let query_str = format!(
        r#"
        SELECT 
            o.id,
            o.name,
            o.owner_user_id,
            u.email as owner_email,
            COALESCE(COUNT(om.user_id), 0) as member_count,
            o.created_at,
            o.updated_at
        FROM organizations o
        INNER JOIN users u ON o.owner_user_id = u.id
        LEFT JOIN organization_members om ON o.id = om.organization_id
        WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
          AND o.is_personal = false
        GROUP BY o.id, o.name, o.owner_user_id, u.email, o.created_at, o.updated_at
        ORDER BY {}
        LIMIT $2 OFFSET $3
        "#,
        order_clause
    );

    let count_query_str = r#"
        SELECT COUNT(DISTINCT o.id) as count
        FROM organizations o
        INNER JOIN users u ON o.owner_user_id = u.id
        WHERE ($1::TEXT IS NULL OR LOWER(o.name) LIKE $1 OR LOWER(u.email) LIKE $1)
          AND o.is_personal = false
    "#;

    let rows = sqlx::query(&query_str)
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let orgs: Vec<EnrichedOrganization> = rows
        .iter()
        .map(|row| <EnrichedOrganization as sqlx::FromRow<sqlx::postgres::PgRow>>::from_row(row))
        .collect::<Result<Vec<_>, _>>()?;

    let count_row = sqlx::query(count_query_str)
        .bind(search_pattern.as_deref())
        .fetch_one(pool)
        .await?;

    let total_count: i64 = count_row.try_get("count")?;

    Ok((orgs, total_count))
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for EnrichedOrganization {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(EnrichedOrganization {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner_user_id: row.try_get("owner_user_id")?,
            owner_email: row.try_get("owner_email")?,
            member_count: row.try_get("member_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
