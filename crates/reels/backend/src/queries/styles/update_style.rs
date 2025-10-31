//! Updates an existing style record in the database.
//!
//! This function modifies a style's name, HTML URL, and screenshot URL. It returns
//! the fully updated `Style` object.

use crate::db::styles::Style;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(skip(tx))]
pub async fn update_style(
    tx: &mut Transaction<'_, Postgres>,
    style_id: Uuid,
    name: &str,
    html_url: &str,
    screenshot_url: &str,
) -> Result<Option<Style>, sqlx::Error> {
    sqlx::query_as!(
        Style,
        r#"
        WITH updated_style AS (
            UPDATE styles 
            SET name = $1, html_url = $2, screenshot_url = $3, updated_at = NOW()
            WHERE id = $4 
            RETURNING id, user_id, name, html_url, screenshot_url, created_at, updated_at
        )
        SELECT 
            u_s.id, u_s.user_id, u_s.name, u_s.html_url, u_s.screenshot_url, 
            u_s.created_at, u_s.updated_at, u.email as creator_email,
            NULL::text AS current_user_access_level
        FROM updated_style u_s
        JOIN users u ON u_s.user_id = u.id
        "#,
        name,
        html_url,
        screenshot_url,
        style_id
    )
    .fetch_optional(&mut **tx)
    .await
}

#[cfg(test)]
mod tests {}