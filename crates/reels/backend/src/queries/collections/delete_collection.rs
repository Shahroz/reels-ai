//! Defines the query for deleting a collection from the database.
//!
//! This function takes a collection ID and deletes the corresponding record
//! from the `collections` table. It returns the result of the query execution.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn delete_collection(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Manually delete associated documents, since ON DELETE SET NULL is used.
    sqlx::query("DELETE FROM documents WHERE collection_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
        
    // Manually delete associated assets, since ON DELETE SET NULL is used.
    sqlx::query("DELETE FROM assets WHERE collection_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Deleting the collection will cascade delete creatives due to ON DELETE CASCADE.
    let result = sqlx::query("DELETE FROM collections WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_delete_collection_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}