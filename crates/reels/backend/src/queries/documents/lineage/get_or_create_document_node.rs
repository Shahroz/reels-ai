//! Document-specific studio node creation and retrieval.
//! 
//! Adapts the existing studio node system to work with documents instead of assets.

use sqlx::types::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct DocumentStudioNodeRow {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub document_id: Uuid,
    pub parent_node_id: Option<Uuid>,
    pub custom_prompt: Option<String>,
    pub metadata: serde_json::Value,
}

/// Get or create a studio node for a document.
/// 
/// Note: We store the document ID in the asset_id field for compatibility 
/// with the existing studio_nodes table structure.
pub async fn get_or_create_document_node(
    pool: &sqlx::PgPool,
    journey_id: Uuid,
    document_id: Uuid,
    parent_node_id: Option<Uuid>,
    custom_prompt: Option<String>,
) -> Result<DocumentStudioNodeRow, sqlx::Error> {
    tracing::info!(
        "🔍 GET_OR_CREATE_DOCUMENT_NODE: journey_id={}, document_id={}, parent_node_id={:?}",
        journey_id, document_id, parent_node_id
    );
    // First, try to find an existing node for this document in this journey
    if let Some(row) = sqlx::query_as!(
        DocumentStudioNodeRow,
        r#"SELECT id, journey_id, asset_id as document_id, parent_node_id, custom_prompt, metadata
           FROM studio_nodes 
           WHERE journey_id = $1 AND asset_id = $2 
           LIMIT 1"#,
        journey_id,
        document_id
    )
    .fetch_optional(pool)
    .await? {
        tracing::info!(
            "✅ FOUND existing node: node_id={}, document_id={}, parent_node_id={:?}",
            row.id, row.document_id, row.parent_node_id
        );
        return Ok(row);
    }

    // Create a new node, storing the document ID in asset_id field
    let created = sqlx::query_as!(
        DocumentStudioNodeRow,
        r#"INSERT INTO studio_nodes (journey_id, asset_id, parent_node_id, custom_prompt, metadata) 
           VALUES ($1, $2, $3, $4, $5) 
           RETURNING id, journey_id, asset_id as document_id, parent_node_id, custom_prompt, metadata"#,
        journey_id,
        document_id,
        parent_node_id,
        custom_prompt,
        serde_json::json!({}) // Default empty metadata
    )
    .fetch_one(pool)
    .await?;

    tracing::info!(
        "✅ CREATED new node: node_id={}, document_id={}, parent_node_id={:?}",
        created.id, created.document_id, created.parent_node_id
    );

    Ok(created)
}

/// Get all nodes for a journey
pub async fn get_document_nodes_for_journey(
    pool: &sqlx::PgPool,
    journey_id: Uuid,
) -> Result<Vec<DocumentStudioNodeRow>, sqlx::Error> {
    tracing::info!("🔍 DB QUERY: get_document_nodes_for_journey journey_id={}", journey_id);
    
    let rows = sqlx::query_as!(
        DocumentStudioNodeRow,
        r#"SELECT id, journey_id, asset_id as document_id, parent_node_id, custom_prompt, metadata
           FROM studio_nodes 
           WHERE journey_id = $1
           ORDER BY created_at ASC"#,
        journey_id
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(
        "✅ DB RESULT: get_document_nodes_for_journey journey_id={}, rows_count={}, rows={:?}",
        journey_id, rows.len(), rows
    );

    Ok(rows)
}

/// Delete a document node
pub async fn delete_document_node(
    pool: &sqlx::PgPool,
    node_id: Uuid,
    journey_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"DELETE FROM studio_nodes 
           WHERE id = $1 AND journey_id = $2"#,
        node_id,
        journey_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
