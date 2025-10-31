//! Fetches a creative by its ID.
//!
//! This query is used when creating a style from an existing creative,
//! to retrieve the source creative's details, such as its `html_url`.

use crate::db::creatives::Creative;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(skip(pool))]
pub async fn find_creative_by_id(
    pool: &PgPool,
    creative_id: Uuid,
) -> Result<Option<Creative>, sqlx::Error> {
    sqlx::query_as!(
        Creative,
        r#"SELECT 
            cr.name, cr.id, cr.collection_id, cr.creative_format_id, cr.style_id, cr.document_ids, 
            cr.asset_ids, cr.html_url, cr.draft_url, cr.screenshot_url, cr.is_published, cr.publish_url, 
            cr.created_at, cr.updated_at, cr.bundle_id
          FROM creatives cr
          LEFT JOIN collections col ON cr.collection_id = col.id
          LEFT JOIN users u ON col.user_id = u.id -- Assuming creator is owner of collection
          WHERE cr.id = $1
        "#,
        creative_id
    )
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {}