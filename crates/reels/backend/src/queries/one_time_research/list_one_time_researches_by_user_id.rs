//! Fetches all one-time research tasks for a specific user.
//!
//! This function retrieves all `one_time_researches` records from the database
//! for a given user, ordered by their creation date in descending order.
//! This allows users to see a history of their one-time research executions.

#[tracing::instrument(skip(pool))]
pub async fn list_one_time_researches_by_user_id(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> Result<std::vec::Vec<crate::db::one_time_research::OneTimeResearch>, sqlx::Error> {
    let researches = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        SELECT * FROM one_time_researches
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    std::result::Result::Ok(researches)
}