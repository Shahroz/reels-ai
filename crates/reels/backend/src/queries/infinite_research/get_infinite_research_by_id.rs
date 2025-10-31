//! Retrieves an infinite research task by its ID for a specific user.
//!
//! This function queries the database for an infinite research task matching
//! the given ID and user ID. It returns an `Option` with the task if found.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool))]
pub async fn get_infinite_research_by_id(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<std::option::Option<crate::db::infinite_research::InfiniteResearch>, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::infinite_research::InfiniteResearch,
        "SELECT * FROM infinite_researches WHERE id = $1 AND user_id = $2",
        id,
        user_id
    );
    let infinite_research = query.fetch_optional(pool).await?;
    std::result::Result::Ok(infinite_research)
}