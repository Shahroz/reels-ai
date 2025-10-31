//! Fetches a one-time research task by its ID for a specific user.
//!
//! This function retrieves a single `one_time_researches` record from the database
//! where the ID and user ID match the provided values.
//! Ensures that users can only access their own research tasks.

#[tracing::instrument(skip(pool))]
pub async fn get_one_time_research_by_id(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        SELECT * FROM one_time_researches
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    );
    let one_time_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(one_time_research)
}