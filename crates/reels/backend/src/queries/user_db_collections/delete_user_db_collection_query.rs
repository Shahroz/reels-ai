//! Executes the database query to delete a user-defined database collection.
//!
//! This function takes a database pool, user ID, and collection ID as input.
//! It attempts to delete the specified collection if it belongs to the user.
//! Returns the number of rows affected (0 or 1) or an SQLx error.
//! Adheres to 'one item per file' and FQN guidelines.

pub async fn delete_user_db_collection_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_to_delete: uuid::Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM user_db_collections
        WHERE id = $1 AND user_id = $2
        "#,
        collection_id_to_delete,
        user_id
    )
    .execute(pool)
    .await?;

    std::result::Result::Ok(result.rows_affected())
}
