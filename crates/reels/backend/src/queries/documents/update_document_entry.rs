use crate::db::document_research_usage::DocumentResearchUsage;

///! Updates an existing document entry in the database.
///!
///! This function modifies the title, content, is_task, and include_research status
///! of a specific document, identified by its ID. It uses COALESCE to only update
///! fields that are provided (not None).
///! It also updates the `updated_at` timestamp to the current time.
///! Returns the fully updated document, with computed fields set to None.
///! This function expects to be called within a transaction where user permissions
///! have already been verified.
pub async fn update_document_entry(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: uuid::Uuid,
    title: Option<&str>,
    content: Option<&str>,
    is_task: Option<bool>,
    include_research: Option<DocumentResearchUsage>,
) -> std::result::Result<crate::db::documents::Document, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        UPDATE documents SET
            title = COALESCE($2, title),
            content = COALESCE($3, content),
            is_task = COALESCE($4, is_task),
            include_research = COALESCE($5, include_research),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id, user_id, title, content, sources, status, created_at, updated_at,
            is_public, is_task, include_research, collection_id
        "#,
        document_id,
        title,
        content,
       is_task,
       include_research as Option<DocumentResearchUsage>
   )
   .fetch_one(&mut **tx)
   .await?;

   Ok(crate::db::documents::Document {
        id: record.id,
        user_id: record.user_id,
        title: record.title,
        content: record.content,
        sources: record.sources,
        status: record.status,
        created_at: record.created_at,
        updated_at: record.updated_at,
        is_public: record.is_public,
        is_task: record.is_task,
        include_research: record
            .include_research
            .map(DocumentResearchUsage::from),
        collection_id: record.collection_id,
    })
}

/// Enhanced version that supports is_public and user_id coordination for admin document management.
///
/// This function extends the basic document update functionality to handle public document
/// creation and management. When is_public is set to true, user_id is set to NULL to make
/// the document globally accessible. When is_public is set to false, user_id is set to the 
/// provided authenticated_user_id to assign ownership. This coordination ensures proper
/// public document behavior while maintaining backward compatibility.
pub async fn update_document_entry_with_visibility(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: uuid::Uuid,
    title: Option<&str>,
    content: Option<&str>,
    is_task: Option<bool>,
    include_research: Option<DocumentResearchUsage>,
    is_public: Option<bool>,
    authenticated_user_id: Option<uuid::Uuid>,
) -> std::result::Result<crate::db::documents::Document, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        UPDATE documents SET
            title = COALESCE($2, title),
            content = COALESCE($3, content),
            is_task = COALESCE($4, is_task),
            include_research = COALESCE($5, include_research),
            is_public = COALESCE($6, is_public),
            user_id = CASE 
                WHEN $6 IS NOT NULL THEN $7
                ELSE user_id 
            END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id, user_id, title, content, sources, status, created_at, updated_at,
            is_public, is_task, include_research, collection_id
        "#,
        document_id,
        title,
        content,
        is_task,
        include_research as Option<DocumentResearchUsage>,
        is_public,
        authenticated_user_id
    )
    .fetch_one(&mut **tx)
    .await?;

    std::result::Result::Ok(crate::db::documents::Document {
        id: record.id,
        user_id: record.user_id,
        title: record.title,
        content: record.content,
        sources: record.sources,
        status: record.status,
        created_at: record.created_at,
        updated_at: record.updated_at,
        is_public: record.is_public,
        is_task: record.is_task,
        include_research: record
            .include_research
            .map(DocumentResearchUsage::from),
        collection_id: record.collection_id,
    })
}

// Per rust_guidelines, tests would typically be defined below.
// #[cfg(test)]
// mod tests {
//     // use super::*; // Not allowed by FQN rule, access items via super::item_name
//
//     #[test]
//     fn test_update_existing_document() {
//         // Setup: mock pool, insert a document
//         // Execute: call super::update_document_entry
//         // Assert: check returned document, check database state
//     }
//
//     #[test]
//     fn test_update_non_existent_document() {
//         // Setup: mock pool
//         // Execute: call super::update_document_entry with a non-existent ID
//         // Assert: check for Ok(None)
//     }
//
//     #[test]
//     fn test_update_document_wrong_user() {
//         // Setup: mock pool, insert a document for user_A
//         // Execute: call super::update_document_entry for user_B
//         // Assert: check for Ok(None)
//     }
// }
