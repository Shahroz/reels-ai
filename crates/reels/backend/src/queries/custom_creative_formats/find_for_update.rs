//! Finds a single custom creative format for update, locking the row.
//!
//! This function fetches a `CustomCreativeFormat` from the database using a `FOR UPDATE` clause.
//! This is to prevent race conditions when the format is being modified.
//! It requires an active database transaction.

pub async fn find_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    format_id: uuid::Uuid,
) -> std::result::Result<
    std::option::Option<crate::db::custom_creative_formats::CustomCreativeFormat>,
    sqlx::Error,
> {
    sqlx::query_as!(
        crate::db::custom_creative_formats::CustomCreativeFormat,
       "SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE id = $1 FOR UPDATE",
       format_id
   )
    .fetch_optional(&mut **tx)
    .await
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}
