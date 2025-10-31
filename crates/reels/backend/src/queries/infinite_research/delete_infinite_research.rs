//! Deletes an infinite research task from the database.
//!
//! This function removes an infinite research task record based on its ID
//! and user ID, returning the number of rows affected.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool))]
pub async fn delete_infinite_research(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM infinite_researches WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(pool)
    .await?;
    std::result::Result::Ok(result.rows_affected())
}