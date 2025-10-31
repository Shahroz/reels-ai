//! Defines the `delete_vocal_tour` database query function.
//!
//! This function deletes a vocal tour from the `vocal_tours` table by its ID.
//! Adheres to the project's Rust coding standards.

pub async fn delete_vocal_tour(
    pool: &sqlx::PgPool,
    vocal_tour_id: uuid::Uuid,
) -> std::result::Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM vocal_tours WHERE id = $1", vocal_tour_id)
        .execute(pool)
        .await
}