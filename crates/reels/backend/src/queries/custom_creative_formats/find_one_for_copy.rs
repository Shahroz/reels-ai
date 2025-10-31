//! Finds a single custom creative format that is available for copying by a user.
//!
//! This function fetches a `CustomCreativeFormat` from the database.
//! It ensures that the format is either public (`is_public = TRUE`) or
//! is owned by the specified user (`user_id = $2`).
//! Admin users can copy any format regardless of ownership or public status.
//! This is used to validate that a user can copy a given format.

pub async fn find_one_for_copy(
    pool: &sqlx::PgPool,
    format_id: uuid::Uuid,
    user_id: uuid::Uuid,
    is_admin: bool,
) -> std::result::Result<crate::db::custom_creative_formats::CustomCreativeFormat, sqlx::Error> {
    if is_admin {
        // Admin users can copy any format
        sqlx::query_as!(
            crate::db::custom_creative_formats::CustomCreativeFormat,
            "SELECT * FROM custom_creative_formats WHERE id = $1",
            format_id
        )
        .fetch_one(pool)
        .await
    } else {
        // Regular users can copy their own formats or any public format
        sqlx::query_as!(
            crate::db::custom_creative_formats::CustomCreativeFormat,
            // The user can copy their own formats or any public format
            "SELECT * FROM custom_creative_formats WHERE id = $1 AND (is_public = TRUE OR user_id = $2)",
            format_id,
            user_id
        )
        .fetch_one(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}