//! Fetches a one-time research task by its ID (internal).
//!
//! This function retrieves a single `one_time_researches` record from the database
//! where the ID matches. It does not check for user_id, as it's intended for
//! internal service-to-service calls.

#[tracing::instrument(skip(pool))]
pub async fn get_one_time_research_by_id_internal(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<crate::db::one_time_research::OneTimeResearch, sqlx::Error> {
    let query = sqlx::query_as!(
        crate::db::one_time_research::OneTimeResearch,
        r#"
        SELECT * FROM one_time_researches
        WHERE id = $1
        "#,
        id
    );
    let one_time_research = query.fetch_one(pool).await?;
    std::result::Result::Ok(one_time_research)
}