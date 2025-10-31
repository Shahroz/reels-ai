//! Fetches a single style by its unique ID.
//!
//! This function retrieves a style's full details from the database,
//! including the creator's email. It's used by the `get_style_by_id` handler.

#[tracing::instrument(skip(pool))]
pub async fn find_style_by_id(pool: &sqlx::PgPool, style_id: uuid::Uuid) -> std::result::Result<std::option::Option<crate::db::styles::Style>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::styles::Style,
        r#"
        SELECT 
            s.id, s.user_id, s.name, s.html_url, s.screenshot_url, 
            s.created_at, s.updated_at
        FROM styles s
        JOIN users u ON s.user_id = u.id
        WHERE s.id = $1
        "#,
        style_id
    )
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {}