//! Retrieves a vocal tour record by its ID.
//!
//! This function queries the vocal_tours table to find a record with the specified
//! vocal tour ID. Returns an Option containing the vocal tour if found, or None if
//! no vocal tour exists with the given ID. Used for vocal tour management operations
//! where the vocal tour ID is known.

#[tracing::instrument(skip(pool))]
pub async fn get_vocal_tour_by_id(
    pool: &sqlx::PgPool,
    vocal_tour_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<crate::db::vocal_tours::VocalTour>, sqlx::Error> {
    let vocal_tour_result = sqlx::query_as!(
        crate::db::vocal_tours::VocalTour,
        "SELECT id, user_id, document_id, asset_ids, created_at, updated_at FROM vocal_tours WHERE id = $1",
        vocal_tour_id
    )
    .fetch_optional(pool)
    .await;

    vocal_tour_result
} 