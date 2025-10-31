//! Retrieves a vocal tour record by its document ID.
//!
//! This function queries the vocal_tours table to find a record with the specified
//! document ID. Returns an Option containing the vocal tour if found, or None if
//! no vocal tour exists for the given document. Used to check if a document has
//! an associated vocal tour for listing integration functionality.

#[tracing::instrument(skip(pool))]
pub async fn get_vocal_tour_by_document_id(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<crate::db::vocal_tours::VocalTour>, sqlx::Error> {
    let vocal_tour_result = sqlx::query_as!(
        crate::db::vocal_tours::VocalTour,
        "SELECT id, user_id, document_id, asset_ids, created_at, updated_at FROM vocal_tours WHERE document_id = $1",
        document_id
    )
    .fetch_optional(pool)
    .await;

    vocal_tour_result
} 