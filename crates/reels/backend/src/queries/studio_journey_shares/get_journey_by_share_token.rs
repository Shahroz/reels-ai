//! Finds an active studio journey share by its public share token.
//!
//! This function is used by the public-facing endpoint to resolve a share
//! token into a journey. It will only return a record if the share token
//! exists and the `is_active` flag is true, ensuring that deactivated
//! links are not accessible.

pub async fn get_journey_by_share_token(
    pool: &sqlx::PgPool,
    share_token: uuid::Uuid,
) -> std::result::Result<Option<crate::db::studio_journey_shares::StudioJourneyShare>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::studio_journey_shares::StudioJourneyShare,
        r#"
        SELECT id, journey_id, share_token, is_active, created_at, updated_at
        FROM studio_journey_shares
        WHERE share_token = $1 AND is_active = TRUE
        "#,
        share_token
    )
    .fetch_optional(pool)
    .await
}