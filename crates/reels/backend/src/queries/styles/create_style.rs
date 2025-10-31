//! Inserts a new style record into the database.
//!
//! This function creates a new style with the provided details, associating it
//! with a user. It returns the newly created `Style` object, including
//! details joined from the `users` table. For public styles, user_id can be NULL.

#[tracing::instrument(skip(pool))]
pub async fn create_style(
    pool: &sqlx::PgPool,
    style_id: uuid::Uuid,
    user_id: std::option::Option<uuid::Uuid>,
    name: &str,
    html_url: &str,
    screenshot_url: &str,
    is_public: bool,
) -> std::result::Result<crate::db::styles::Style, sqlx::Error> {
    sqlx::query_as!(
        crate::db::styles::Style,
        r#"
        WITH inserted_style AS (
            INSERT INTO styles (id, user_id, name, html_url, screenshot_url, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
        )
        SELECT 
            i_s.id, i_s.user_id, i_s.name, i_s.html_url, i_s.screenshot_url, i_s.is_public,
            i_s.created_at, i_s.updated_at
        FROM inserted_style i_s
        LEFT JOIN users u ON i_s.user_id = u.id
        "#,
        style_id,
        user_id,
        name,
        html_url,
        screenshot_url,
        is_public
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {}