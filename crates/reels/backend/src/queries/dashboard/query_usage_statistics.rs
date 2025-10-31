#![allow(clippy::disallowed_methods)]
//! Defines queries for fetching user-specific or global usage statistics.
//!
//! This module contains functions to query the database for counts of
//! various entities created by users, supporting filtering, pagination, and sorting.
//!
//! Revision History
//! - 2025-06-18T17:52:23Z @USER: Refactored from usage_statistics.rs route handler.


/// Represents usage statistics for a single user.
#[derive(serde::Serialize, Debug, utoipa::ToSchema, sqlx::FromRow)]
pub struct UsageStatisticsItem {
   #[schema(format = "uuid", value_type=String, example = "550e8400-e29b-41d4-a716-446655440000")]
   pub user_id: uuid::Uuid,
   #[schema(example = "user@example.com")]
   pub user_email: std::string::String,
   #[schema(value_type = String, format = "date-time", example = "2021-01-01T00:00:00Z")]
   pub user_created_at: chrono::DateTime<chrono::Utc>,
   pub user_status: std::string::String,
   #[schema(example = "trial")]
   pub subscription_status: std::string::String,
   pub styles_count: Option<i64>,
   pub assets_count: Option<i64>,
   pub documents_count: Option<i64>,
   #[schema(example = "10")]
   pub creatives_count: Option<i64>,
   #[schema(example = "10")]
   pub custom_formats_count: Option<i64>,
}

#[tracing::instrument(
    name = "query_usage_statistics_items",
    skip(pool),
    fields(
        start_date = %start_date.map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string()),
        end_date = %end_date.map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string()),
        email_filter = %email_filter_value.as_deref().unwrap_or("N/A"),
        organization_id_filter = %organization_id_filter.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()),
        sort_by = %sort_by,
        sort_order = %sort_order,
        limit = %limit,
        offset = %offset,
    )
)]
pub async fn query_usage_statistics_items(
    pool: &sqlx::PgPool,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    email_filter_value: Option<String>,
    organization_id_filter: Option<uuid::Uuid>,
    subscription_status_filter: Option<String>,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<std::vec::Vec<UsageStatisticsItem>, sqlx::Error> {
    let sort_by_sql_literal = match sort_by {
        "user_email" => "u.email",
        "user_created_at" => "u.created_at",
        "styles_count" => "styles_count",
        "assets_count" => "assets_count",
        "documents_count" => "documents_count",
        "creatives_count" => "creatives_count",
        "custom_formats_count" => "custom_formats_count",
        _ => "u.email",
    };
    let sort_order_sql_literal = if sort_order == "desc" { "DESC" } else { "ASC" };

    // Parse subscription status filter string into vector
    let subscription_status_vec: Option<Vec<String>> = subscription_status_filter
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect());

    // Base query with time-based filtering
    let base_query = r#"
            SELECT
               u.id AS user_id,
               u.email AS user_email,
               u.status AS user_status,
               u.subscription_status AS subscription_status,
               u.created_at as user_created_at,
               (SELECT COUNT(*) FROM styles s WHERE s.user_id = u.id AND ($1::TIMESTAMPTZ IS NULL OR s.created_at >= $1) AND ($2::TIMESTAMPTZ IS NULL OR s.created_at < $2)) AS styles_count,
               (SELECT COUNT(*) FROM assets a WHERE a.user_id = u.id AND ($1::TIMESTAMPTZ IS NULL OR a.created_at >= $1) AND ($2::TIMESTAMPTZ IS NULL OR a.created_at < $2)) AS assets_count,
               (SELECT COUNT(*) FROM documents d WHERE d.user_id = u.id AND ($1::TIMESTAMPTZ IS NULL OR d.created_at >= $1) AND ($2::TIMESTAMPTZ IS NULL OR d.created_at < $2)) AS documents_count,
               (SELECT COUNT(*) FROM creatives c INNER JOIN collections col ON c.collection_id = col.id WHERE col.user_id = u.id AND ($1::TIMESTAMPTZ IS NULL OR c.created_at >= $1) AND ($2::TIMESTAMPTZ IS NULL OR c.created_at < $2)) AS creatives_count,
               (SELECT COUNT(*) FROM custom_creative_formats ccf WHERE ccf.user_id = u.id AND ($1::TIMESTAMPTZ IS NULL OR ccf.created_at >= $1) AND ($2::TIMESTAMPTZ IS NULL OR ccf.created_at < $2)) AS custom_formats_count
            FROM users u
        "#;

    // Add organization joins based on whether we're filtering by organization
    let organization_joins = if organization_id_filter.is_some() {
        // When filtering by organization, use INNER JOIN to only get users in that organization
        " INNER JOIN organization_members om ON u.id = om.user_id INNER JOIN organizations o ON om.organization_id = o.id"
    } else {
        ""
    };
    let query_string = format!("{} {}", base_query, organization_joins);

    // Build conditions for additional filters
    let mut conditions = Vec::new();
    let mut param_count = 3; // start_date, end_date, limit

    if email_filter_value.is_some() {
        conditions.push(format!("u.email ILIKE ${param_count}"));
        param_count += 1;
    }

    if subscription_status_vec.is_some() {
        conditions.push(format!("u.subscription_status = ANY(${param_count})"));
        param_count += 1;
    }

    if organization_id_filter.is_some() {
        conditions.push(format!("om.organization_id = ${param_count}"));
        param_count += 1;
    }

    // Add WHERE clause if conditions exist
    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // Complete query with WHERE, ORDER BY, LIMIT, and OFFSET
    let complete_query = format!(
        "{}{} ORDER BY {} {} LIMIT ${} OFFSET ${}",
        query_string,
        where_clause,
        sort_by_sql_literal,
        sort_order_sql_literal,
        param_count,
        param_count + 1
    );

    let mut query = sqlx::query_as::<_, UsageStatisticsItem>(&complete_query)
        .bind(start_date)
        .bind(end_date);

    if let Some(email) = email_filter_value {
        query = query.bind(format!("%{email}%"));
    }

    if let Some(subscription_statuses) = subscription_status_vec {
        query = query.bind(subscription_statuses);
    }

    if let Some(organization_id) = organization_id_filter {
        query = query.bind(organization_id);
    }

    query.bind(limit).bind(offset).fetch_all(pool).await
}

#[tracing::instrument(
    name = "query_usage_statistics_total_count",
    skip(pool),
    fields(
        email_filter = %email_filter_value.as_deref().unwrap_or("N/A"),
        organization_id_filter = %organization_id_filter.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string())
    )
)]
pub async fn query_usage_statistics_total_count(
    pool: &sqlx::PgPool,
    email_filter_value: Option<String>,
    organization_id_filter: Option<uuid::Uuid>,
    subscription_status_filter: Option<String>,
) -> Result<i64, sqlx::Error> {
    type TotalCount = crate::sql_utils::count_sql_results::TotalCount;
    
    // Parse subscription status filter string into vector
    let subscription_status_vec: Option<Vec<String>> = subscription_status_filter
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect());
    
    let mut query_string = String::from("SELECT COUNT(*) AS count FROM users u");
    let mut conditions = Vec::new();
    let mut param_count = 1;
    
    // Add organization join based on whether we're filtering by organization
    if organization_id_filter.is_some() {
        query_string.push_str(" INNER JOIN organization_members om ON u.id = om.user_id");
    }
    
    if email_filter_value.is_some() {
        conditions.push(format!("u.email ILIKE ${param_count}"));
        param_count += 1;
    }
    
    if subscription_status_vec.is_some() {
        conditions.push(format!("u.subscription_status = ANY(${param_count})"));
        param_count += 1;
    }
    
    if organization_id_filter.is_some() {
        conditions.push(format!("om.organization_id = ${param_count}"));
        param_count += 1;
    }
    
    if !conditions.is_empty() {
        query_string.push_str(" WHERE ");
        query_string.push_str(&conditions.join(" AND "));
    }
    
    let mut query = sqlx::query_as::<_, TotalCount>(&query_string);
    
    if let Some(ref email) = email_filter_value {
        query = query.bind(format!("%{email}%"));
    }
    
    if let Some(subscription_statuses) = subscription_status_vec {
        query = query.bind(subscription_statuses);
    }
    
    if let Some(organization_id) = organization_id_filter {
        query = query.bind(organization_id);
    }
    
    let total_count_result = query.fetch_one(pool).await?;
    Ok(total_count_result.count.unwrap_or_default())
}
