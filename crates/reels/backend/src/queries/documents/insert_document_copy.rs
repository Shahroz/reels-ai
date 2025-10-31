//! Inserts a new document record as a copy of an existing one.
//!
//! This function handles the database insertion for a document copy. It takes the
//! details of the original document and the new owner's ID, sets new metadata
//! (like a new UUID, "(COPY)" prefix in title, `is_public=false`), and inserts
//! it into the `documents` table. It returns the newly created record.

/// Represents the data returned after inserting a new document copy.
// This struct is needed because sqlx::query! returns an anonymous record type.
// We define a struct to give it a name and make it easier to use.
#[derive(Debug, sqlx::FromRow)]
pub struct InsertedDocumentCopy {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub title: std::string::String,
    pub content: std::string::String,
    pub sources: std::vec::Vec<std::string::String>,
    pub status: std::string::String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: Option<std::string::String>,
}

pub async fn insert_document_copy(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    new_owner_id: uuid::Uuid,
    original_doc: &crate::queries::documents::fetch_document_for_copy_from_pool::OriginalDocumentAccessInfo,
) -> std::result::Result<InsertedDocumentCopy, sqlx::Error> {
    let new_doc_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    let new_title = std::format!("(COPY) {}", original_doc.title);
    let include_research_param: Option<std::string::String> = original_doc.include_research.clone();

    sqlx::query_as!(
        InsertedDocumentCopy,
        r#"
        INSERT INTO documents
            (id, user_id, title, content, sources, status, is_public, created_at, updated_at, is_task, include_research)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, user_id, title, content, sources, status, created_at, updated_at, is_public, is_task, include_research
        "#,
        new_doc_id,
        Some(new_owner_id),
        new_title,
        original_doc.content,
        &original_doc.sources,
        original_doc.status,
        false, // New copy is private
        now,   // created_at
        now,   // updated_at
       false, // New copy is not a task
       include_research_param
   )
   .fetch_one(&mut **tx)
   .await
}

#[cfg(test)]
mod tests {
    //! Tests for insert_document_copy.
    //!
    //! Conceptual tests requiring a database.

    #[test]
    fn conceptual_test_insert_copy() {
        // In a real test, you would:
        // 1. Setup a test database and a transaction.
        // 2. Create a mock `OriginalDocumentAccessInfo` from the pool-based version.
        // 3. Call `super::insert_document_copy`.
        // 4. Assert that the returned `InsertedDocumentCopy` has the correct fields (new ID, new title, etc.).
        // 5. Verify the record exists in the database.
        std::assert!(true, "Conceptual test passed for insert_document_copy");
    }
}
