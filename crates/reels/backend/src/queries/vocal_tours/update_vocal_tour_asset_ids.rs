//! Updates the asset IDs array for an existing vocal tour record.
//!
//! This function modifies the asset_ids field of a vocal tour record identified
//! by its ID. It replaces the existing asset IDs with the provided array and
//! updates the updated_at timestamp. Returns the fully updated vocal tour record.
//! Used when assets need to be added or modified after initial vocal tour creation.

#[tracing::instrument(skip(pool))]
pub async fn update_vocal_tour_asset_ids(
    pool: &sqlx::PgPool,
    vocal_tour_id: uuid::Uuid,
    user_id: uuid::Uuid,
    asset_ids: &[uuid::Uuid],
) -> std::result::Result<crate::db::vocal_tours::VocalTour, sqlx::Error> {
    let vocal_tour = sqlx::query_as!(
        crate::db::vocal_tours::VocalTour,
        r#"
        UPDATE vocal_tours
        SET
            asset_ids = $3,
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, document_id, asset_ids, created_at, updated_at
        "#,
        vocal_tour_id,
        user_id,
        asset_ids
    )
    .fetch_one(pool)
    .await?;
    
    std::result::Result::Ok(vocal_tour)
} 