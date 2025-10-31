//! Checks if a custom creative format exists in the database by its ID.
//!
//! This function queries for the existence of a format by its primary key.
//! It does not check for ownership.
//! Returns true if a format with the given ID exists, false otherwise.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn exists(
    pool: &sqlx::PgPool,
    format_id: uuid::Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM custom_creative_formats WHERE id = $1")
        .bind(format_id)
        .fetch_one(pool)
        .await?;
    std::result::Result::Ok(count > 0)
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}