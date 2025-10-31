//! Creates a new one-time research task in the database.
//!
//! This function inserts a new record into the `one_time_researches` table
//! with the provided details. It returns the newly created task upon success.
//! Follows the one-item-per-file guideline.

#[tracing::instrument(skip(pool, prompt))]
pub async fn create_one_time_research(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    prompt: &str,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        INSERT INTO one_time_researches (user_id, prompt)
        VALUES ($1, $2)
        RETURNING *
        "#,
        user_id,
        prompt
    );
    let one_time_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(one_time_research)
}