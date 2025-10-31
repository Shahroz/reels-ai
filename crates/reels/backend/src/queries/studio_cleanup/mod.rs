//! Studio cleanup functionality for assets and documents
//! 
//! This module provides functions to clean up studio relationships (nodes, edges, journeys)
//! when assets or documents are deleted, while preserving the actual derived content.

use sqlx::PgPool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub deleted_nodes: i64,
    pub deleted_orphaned_nodes: i64,
    pub deleted_edges: i64,
    pub deleted_journeys: i64,
}

/// Find all orphaned studio nodes that will lose their parent
pub async fn find_orphaned_studio_nodes(
    pool: &PgPool,
    entity_id: Uuid,
    entity_type: &str,
) -> Result<Vec<OrphanedNode>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH RECURSIVE descendants AS (
            -- Base case: direct children from provenance edges
            SELECT pe.target_id as descendant_id, 1 as depth
            FROM provenance_edges pe
            WHERE pe.source_id = $1 
              AND pe.source_type = $2
              AND pe.target_type = $2
            
            UNION ALL
            
            -- Recursive case: children of descendants
            SELECT pe.target_id as descendant_id, d.depth + 1
            FROM provenance_edges pe
            INNER JOIN descendants d ON pe.source_id = d.descendant_id
            WHERE pe.source_type = $2 AND pe.target_type = $2
        )
        SELECT sn.id as node_id, sn.journey_id, sn.asset_id, d.depth
        FROM descendants d
        INNER JOIN studio_nodes sn ON sn.asset_id = d.descendant_id
        "#,
        entity_id,
        entity_type
    )
    .fetch_all(pool)
    .await?;
    
    let nodes = rows.into_iter().filter_map(|row| {
        Some(OrphanedNode {
            node_id: row.node_id,
            journey_id: row.journey_id,
            asset_id: row.asset_id,
            depth: row.depth?,
        })
    }).collect();
    
    Ok(nodes)
}

/// Clean up studio relationships for deleted entity (preserves derived assets/documents)
pub async fn cleanup_studio_relationships(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    entity_id: Uuid,
    entity_type: &str,
) -> Result<CleanupSummary, sqlx::Error> {
    // 1. First, find and delete all descendant studio nodes using provenance edges
    let deleted_orphaned_nodes = sqlx::query!(
        r#"
        DELETE FROM studio_nodes WHERE asset_id IN (
            WITH RECURSIVE descendants AS (
                -- Base case: direct children from provenance edges
                SELECT pe.target_id as descendant_id
                FROM provenance_edges pe
                WHERE pe.source_id = $1 
                  AND pe.source_type = $2
                  AND pe.target_type = $2
                
                UNION ALL
                
                -- Recursive case: children of descendants
                SELECT pe.target_id as descendant_id
                FROM provenance_edges pe
                INNER JOIN descendants d ON pe.source_id = d.descendant_id
                WHERE pe.source_type = $2 AND pe.target_type = $2
            )
            SELECT descendant_id FROM descendants
        )
        "#,
        entity_id,
        entity_type
    )
    .execute(&mut **tx)
    .await?
    .rows_affected() as i64;
    
    // 2. Now delete all studio nodes referencing this entity directly
    let deleted_nodes = sqlx::query!(
        "DELETE FROM studio_nodes WHERE asset_id = $1",
        entity_id
    )
    .execute(&mut **tx)
    .await?
    .rows_affected() as i64;
    
    // 3. Delete all provenance edges where this entity is source or target
    let deleted_edges = sqlx::query!(
        "DELETE FROM provenance_edges 
         WHERE (source_type = $1 AND source_id = $2) 
            OR (target_type = $1 AND target_id = $2)",
        entity_type,
        entity_id
    )
    .execute(&mut **tx)
    .await?
    .rows_affected() as i64;
    
    // 4. Delete studio journeys that reference this entity as root
    // When the root asset is deleted, the entire journey becomes invalid
    let updated_journeys = sqlx::query!(
        "DELETE FROM studio_journeys 
         WHERE root_asset_id = $1",
        entity_id
    )
    .execute(&mut **tx)
    .await?
    .rows_affected() as i64;
    
    Ok(CleanupSummary {
        deleted_nodes,
        deleted_orphaned_nodes,
        deleted_edges,
        deleted_journeys: updated_journeys,
    })
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct OrphanedNode {
    pub node_id: Uuid,
    pub journey_id: Uuid,
    pub asset_id: Uuid,
    pub depth: i32,
}
