//! Defines the query for fetching baseline entity counts.
//!
//! This module contains the function to query the database for the total count
//! of entities that existed before a specified date. This is used for cumulative
//! chart calculations where we need a starting baseline.
//!
//! Revision History
//! - 2025-10-06 @AI: Initial creation for cumulative chart baseline support.

/// Fetches the baseline count of entities that existed before a given date.
#[tracing::instrument(
    name = "query_baseline_count",
    skip(pool),
    fields(
        entity_type = %entity_type,
        before_date = %before_datetime,
        user_email = %user_email.as_deref().unwrap_or("N/A")
    )
)]
pub async fn query_baseline_count(
    pool: &sqlx::PgPool,
    entity_type: crate::routes::dashboard::daily_activity_stats::ActivityEntityType,
    before_datetime: chrono::DateTime<chrono::Utc>,
    user_email: Option<String>,
) -> Result<i64, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("");
    let main_table_alias = "e"; // e for entity

    // Base SELECT
    query_builder.push(format!("SELECT COUNT(*) as count "));

    // FROM and JOIN clauses
    match entity_type {
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users => { 
            query_builder.push(format!("FROM users {main_table_alias} ")); 
        }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Documents => { 
            query_builder.push(format!("FROM documents {main_table_alias} ")); 
        }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Styles => { 
            query_builder.push(format!("FROM styles {main_table_alias} ")); 
        }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Assets => { 
            query_builder.push(format!("FROM assets {main_table_alias} ")); 
        }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::CustomCreativeFormats => { 
            query_builder.push(format!("FROM custom_creative_formats {main_table_alias} ")); 
        }
        crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives => {
            query_builder.push(format!("FROM creatives {main_table_alias} "));
            if user_email.is_some() {
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

    // Date filtering - only include entities created before the start date
    add_where_connector(&mut query_builder);
    let before_condition_sql = format!("{main_table_alias}.created_at < ");
    query_builder.push(&before_condition_sql);
    query_builder.push_bind(before_datetime);

    // User email filtering
    if let Some(ref email) = user_email {
        add_where_connector(&mut query_builder);
        match entity_type {
            crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives => {
                query_builder.push("u.email = ");
            }
            crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users => {
                query_builder.push(format!("{main_table_alias}.email = "));
            }
            _ => {
                query_builder.push(format!("{main_table_alias}.user_id = (SELECT id FROM users WHERE email = "));
            }
        }
        query_builder.push_bind(email.clone());
        if !matches!(
            entity_type, 
            crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Creatives 
            | crate::routes::dashboard::daily_activity_stats::ActivityEntityType::Users
        ) {
            query_builder.push(") ");
        }
    }

    let query_str = query_builder.sql();
    log::debug!("Executing SQL for baseline count: {}", query_str);

    #[derive(sqlx::FromRow)]
    struct CountResult {
        count: Option<i64>,
    }

    let result: CountResult = query_builder
        .build_query_as()
        .fetch_one(pool)
        .await?;

    Ok(result.count.unwrap_or(0))
}

