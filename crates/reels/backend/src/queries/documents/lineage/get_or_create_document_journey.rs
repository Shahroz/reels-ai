//! Document-specific studio journey creation and retrieval.
//! 
//! Adapts the existing studio journey system to work with documents instead of assets.
//! This allows the Content Studio to reuse the same journey infrastructure as the Image Studio.

use sqlx::types::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct DocumentStudioJourneyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub root_document_id: Option<Uuid>,
    pub name: Option<String>,
}

/// Get or create a studio journey for a document.
/// 
/// Note: We store the document ID in the root_asset_id field for compatibility 
/// with the existing studio_journeys table structure. This avoids requiring
/// schema changes while enabling document support.
pub async fn get_or_create_document_journey(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    root_document_id: Uuid,
    name: Option<String>,
) -> Result<DocumentStudioJourneyRow, sqlx::Error> {
    // First, try to find an existing journey for this document
    if let Some(row) = sqlx::query_as!(
        DocumentStudioJourneyRow,
        r#"SELECT id, user_id, root_asset_id as root_document_id, name 
           FROM studio_journeys 
           WHERE user_id = $1 AND root_asset_id = $2 
           LIMIT 1"#,
        user_id,
        root_document_id
    )
    .fetch_optional(pool)
    .await? {
        return Ok(row);
    }

    // Create a new journey, storing the document ID in root_asset_id field
    let journey_name = name.unwrap_or_else(|| "Content Studio Journey".to_string());
    let created = sqlx::query_as!(
        DocumentStudioJourneyRow,
        r#"INSERT INTO studio_journeys (user_id, root_asset_id, name) 
           VALUES ($1, $2, $3) 
           RETURNING id, user_id, root_asset_id as root_document_id, name"#,
        user_id,
        root_document_id,
        journey_name
    )
    .fetch_one(pool)
    .await?;

    Ok(created)
}

/// Get an existing document journey by ID
pub async fn get_document_journey(
    pool: &sqlx::PgPool,
    journey_id: Uuid,
    user_id: Uuid,
) -> Result<Option<DocumentStudioJourneyRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        DocumentStudioJourneyRow,
        r#"SELECT id, user_id, root_asset_id as root_document_id, name 
           FROM studio_journeys 
           WHERE id = $1 AND user_id = $2"#,
        journey_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Update journey name
pub async fn update_document_journey_name(
    pool: &sqlx::PgPool,
    journey_id: Uuid,
    user_id: Uuid,
    name: String,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE studio_journeys 
           SET name = $1, updated_at = NOW() 
           WHERE id = $2 AND user_id = $3"#,
        name,
        journey_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
