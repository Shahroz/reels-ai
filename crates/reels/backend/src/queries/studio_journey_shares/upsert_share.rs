//! Creates or reactivates a share for a Studio Journey.
//!
//! This function performs an "upsert" operation. If a share for the given
//! journey already exists, it updates `is_active` to `true` and touches the
//! `updated_at` timestamp. If no share exists, it creates a new one.
//! Returns the created or updated `StudioJourneyShare` record.

pub async fn upsert_share(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    journey_id: uuid::Uuid,
) -> std::result::Result<crate::db::studio_journey_shares::StudioJourneyShare, sqlx::Error> {
    sqlx::query_as!(
        crate::db::studio_journey_shares::StudioJourneyShare,
        r#"
        INSERT INTO studio_journey_shares (journey_id)
        VALUES ($1)
        ON CONFLICT (journey_id)
        DO UPDATE SET is_active = TRUE, updated_at = NOW()
        RETURNING id, journey_id, share_token, is_active, created_at, updated_at
        "#,
        journey_id
    )
    .fetch_one(&mut **tx)
    .await
}