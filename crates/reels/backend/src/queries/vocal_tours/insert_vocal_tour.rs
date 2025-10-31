//! Inserts a new vocal tour record into the database.
//!
//! This function creates a new vocal tour entity that links a document with its
//! generated assets. It takes the user ID, document ID, and asset IDs as parameters
//! and returns the newly created vocal tour record. Used during the vocal tour
//! creation workflow to establish the relationship between outputs.

#[tracing::instrument(skip(tx))]
pub async fn insert_vocal_tour(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    document_id: uuid::Uuid,
    asset_ids: &[uuid::Uuid],
) -> std::result::Result<crate::db::vocal_tours::VocalTour, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::vocal_tours::VocalTour,
        r#"
        INSERT INTO vocal_tours (user_id, document_id, asset_ids)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, document_id, asset_ids, created_at, updated_at
        "#,
        user_id,
        document_id,
        asset_ids
    );
    let vocal_tour = query.fetch_one(&mut **tx).await?;
    std::result::Result::Ok(vocal_tour)
} 