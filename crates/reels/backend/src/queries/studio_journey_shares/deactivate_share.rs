//! Deactivates a share link for a Studio Journey.
//!
//! This function performs a soft delete by setting the `is_active` flag to `false`
//! for the share associated with the given `journey_id`. This preserves the
//! share link and token for possible future reactivation.
//! Returns the number of rows affected (should be 1 on success).

pub async fn deactivate_share(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    journey_id: uuid::Uuid,
) -> std::result::Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE studio_journey_shares
        SET is_active = FALSE, updated_at = NOW()
        WHERE journey_id = $1
        "#,
        journey_id
    )
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}