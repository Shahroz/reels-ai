#![allow(clippy::disallowed_methods)]
//! Inserts a new document entry into the database with extended details.
//!
//! This function handles the insertion of a new document, including fields
//! for public status, task status, and research usage options. It returns
//! the core data of the newly created document record.

use crate::db::document_research_usage::DocumentResearchUsage;

/// Represents the data returned directly from the database after insertion.
#[derive(Debug, serde::Serialize, Clone, PartialEq)]
pub struct InsertedDocumentData {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub title: String,
    pub content: String,
    pub sources: Vec<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: Option<String>,
    pub collection_id: Option<uuid::Uuid>,
}

pub async fn insert_document_entry(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Option<uuid::Uuid>, // The user_id to be stored in the document (can be None for public)
    title: &str,
    content: &str,
    sources: &[String],
    is_public: bool,
    is_task: bool,
    include_research: Option<DocumentResearchUsage>,
    collection_id: Option<uuid::Uuid>, // Optional collection ID to attach the document to
) -> std::result::Result<InsertedDocumentData, sqlx::Error> {
    sqlx::query_as!(
        InsertedDocumentData,
        r#"
        INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research, collection_id)
        VALUES ($1, $2, $3, $4, 'Pending', $5, $6, $7, $8)
        RETURNING
            id, user_id, title, content, sources, status, created_at, updated_at,
            is_public, is_task, include_research::TEXT as "include_research?", collection_id
        "#,
        user_id,
        title,
        content,
        sources,
        is_public,
        is_task,
        include_research as Option<DocumentResearchUsage>,
        collection_id
   )
   .fetch_one(&mut **tx)
   .await
}
