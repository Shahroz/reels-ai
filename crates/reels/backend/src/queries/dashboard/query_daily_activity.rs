//! Defines the query for fetching daily activity statistics.
//!
//! This module contains the function and data structures necessary
//! to query the database for daily counts of various entities
//! within a specified date range and optional user filter.
//!
//! Revision History
//! - 2025-06-18T17:52:23Z @USER: Refactored from daily_activity_stats.rs route handler.

/// Represents a daily count for an entity.
#[derive(sqlx::FromRow, Debug)]
pub struct DailyCount {
    pub activity_date: chrono::NaiveDate,
    pub count: i64,
}

/// Fetches daily activity counts for a given entity type from the database.
#[tracing::instrument(
    name = "query_daily_activity",
    skip(pool),
    fields(
        entity_type = %entity_type,
        start_date = %start_datetime,
        end_date = %end_datetime,
        user_email = %user_email.as_deref().unwrap_or("N/A")
    )
)]
pub async fn query_daily_activity(
    pool: &sqlx::PgPool,
    entity_type: crate::routes::dashboard::daily_activity_stats::ActivityEntityType,
    start_datetime: chrono::DateTime<chrono::Utc>,
    end_datetime: chrono::DateTime<chrono::Utc>,
    user_email: Option<String>,
) -> Result<std::vec::Vec<DailyCount>, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("");
    let main_table_alias = "e"; // e for entity

    // Base SELECT and GROUP BY
    query_builder.push(format!("SELECT DATE({main_table_alias}.created_at) as activity_date, COUNT(*) as count "));

    // FROM and JOIN clauses
    match entity_type {
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users => { query_builder.push(format!("FROM users {main_table_alias} ")); }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Documents => { query_builder.push(format!("FROM documents {main_table_alias} ")); }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Styles => { query_builder.push(format!("FROM styles {main_table_alias} ")); }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Assets => { query_builder.push(format!("FROM assets {main_table_alias} ")); }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::CustomCreativeFormats => { query_builder.push(format!("FROM custom_creative_formats {main_table_alias} ")); }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives => {
            query_builder.push(format!("FROM creatives {main_table_alias} "));
            if user_email.is_some() { // Only join collections and users if filtering by user
                query_builder.push(format!("INNER JOIN collections col ON {main_table_alias}.collection_id = col.id "));
                query_builder.push("INNER JOIN users u ON col.user_id = u.id ");
            }
        },
    }
    
    let mut first_where_clause = true;
    let mut add_where_connector = |qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>| {
        if first_where_clause {
            qb.push("WHERE ");
            first_where_clause = false;
        } else {
            qb.push(" AND ");
        }
    };

   // Date filtering
   add_where_connector(&mut query_builder);
   let start_condition_sql = format!("{main_table_alias}.created_at >= ");
   query_builder.push(&start_condition_sql);
   query_builder.push_bind(start_datetime);

   add_where_connector(&mut query_builder);
   let end_condition_sql = format!("{main_table_alias}.created_at < ");
   query_builder.push(&end_condition_sql);
   query_builder.push_bind(end_datetime);

   // User email filtering
    if let Some(ref email) = user_email {
        add_where_connector(&mut query_builder);
        match entity_type {
            crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives => { // Already joined with users table aliased as 'u'
                query_builder.push("u.email = ");
            }
            crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users => { // Filtering the users table itself
                 query_builder.push(format!("{main_table_alias}.email = "));
            }
            _ => { // For entities that have a user_id foreign key to users table
                // A subquery is used here.
                query_builder.push(format!("{main_table_alias}.user_id = (SELECT id FROM users WHERE email = "));
            }
        }
        query_builder.push_bind(email.clone());
        // Add closing parenthesis for the subquery if it was used
        if !matches!(entity_type, crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives | crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users) {
             // This covers Documents, Styles, Assets, CustomCreativeFormats which use the subquery
            query_builder.push(") ");
        }
    }
    
    query_builder.push(format!(" GROUP BY DATE({main_table_alias}.created_at) ORDER BY activity_date ASC"));

    let query_str = query_builder.sql();
    log::debug!("Executing SQL for daily activity: {}", query_str);

    query_builder
        .build_query_as()
        .fetch_all(pool)
        .await
}