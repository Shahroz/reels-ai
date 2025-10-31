//! Deletes a document entry from the database by its ID and user ID.
//!
//! This function executes a SQL DELETE statement to remove a specific document
//! associated with a given user. It ensures that only the owner can delete
//! their document. Now includes automatic studio cleanup to remove orphaned studio nodes 
//! and provenance edges. The function returns the number of rows affected upon successful deletion,
//! or an SQLx error if the database operation fails.
//!
//! Revision History
//! - 2025-05-15T14:19:14Z @AI: Created function to encapsulate document deletion logic.
//! - 2025-09-29T00:00:00Z @AI: Added automatic studio cleanup functionality.

use crate::queries::studio_cleanup::cleanup_studio_relationships;

pub async fn delete_document_entry(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // 1. Clean up studio relationships first (always)
    let _cleanup_summary = cleanup_studio_relationships(&mut tx, document_id, "document").await?;
    
    // 2. Delete the main document (with user permission check)
    let result = sqlx::query("DELETE FROM documents WHERE id = $1 AND user_id = $2")
        .bind(document_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(result.rows_affected())
}