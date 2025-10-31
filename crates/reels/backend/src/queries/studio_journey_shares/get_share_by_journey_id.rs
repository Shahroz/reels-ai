//! Finds a studio journey share record by its journey ID.
//!
//! This function is used to retrieve the sharing status and token for a
//! journey, typically for the journey's owner. It fetches the share
//! record regardless of its `is_active` status.

pub async fn get_share_by_journey_id(
    pool: &sqlx::PgPool,
    journey_id: uuid::Uuid,
) -> std::result::Result<Option<crate::db::studio_journey_shares::StudioJourneyShare>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::studio_journey_shares::StudioJourneyShare,
        r#"
        SELECT id, journey_id, share_token, is_active, created_at, updated_at
        FROM studio_journey_shares
        WHERE journey_id = $1
        "#,
        journey_id
    )
    .fetch_optional(pool)
    .await
}