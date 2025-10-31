//! Deletes a custom creative format from the database owned by a specific user.
//!
//! This function executes a `DELETE` statement on the `custom_creative_formats` table.
//! It targets a format by its ID and ensures it's owned by the provided user ID.
//! Admin users can delete any format regardless of ownership.
//! Returns the number of rows affected, which should be 1 on success.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn delete(
    pool: &sqlx::PgPool,
    format_id: uuid::Uuid,
    user_id: uuid::Uuid,
    is_admin: bool,
) -> std::result::Result<u64, sqlx::Error> {
    let result = if is_admin {
        // Admin users can delete any format
        sqlx::query!(
            "DELETE FROM custom_creative_formats WHERE id = $1",
            format_id
        )
        .execute(pool)
        .await?
    } else {
        // Regular users can only delete their own formats
        sqlx::query!(
            "DELETE FROM custom_creative_formats WHERE id = $1 AND user_id = $2",
            format_id,
            user_id
        )
        .execute(pool)
        .await?
    };

    std::result::Result::Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}