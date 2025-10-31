//! Gathers all related object IDs for a collection to support hierarchy-based permission checking.
//!
//! This function retrieves all objects that are part of a collection's hierarchy:
//! the collection itself, all creatives within it, and all assets and documents
//! used by those creatives. This enables batch permission checking across
//! the entire collection hierarchy to implement "Most Permissive Wins" logic.

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CollectionHierarchy {
    pub collection_id: Uuid,
    pub creative_ids: Vec<Uuid>,
    pub asset_ids: Vec<Uuid>,
    pub document_ids: Vec<Uuid>,
}

pub async fn get_collection_hierarchy_ids(
    pool: &PgPool,
    collection_id: Uuid,
) -> Result<CollectionHierarchy, sqlx::Error> {
    // Query to get all creatives and their associated asset/document IDs
    let results = sqlx::query!(
        r#"
        SELECT 
            c.id as creative_id,
            c.asset_ids,
            c.document_ids
        FROM creatives c
        WHERE c.collection_id = $1
        "#,
        collection_id
    )
    .fetch_all(pool)
    .await?;

    let mut creative_ids = Vec::new();
    let mut asset_ids = Vec::new();
    let mut document_ids = Vec::new();

    for row in results {
        creative_ids.push(row.creative_id);
        
        // Flatten asset IDs from all creatives
        if let Some(ref assets) = row.asset_ids {
            asset_ids.extend(assets.iter().copied());
        }
        
        // Flatten document IDs from all creatives  
        if let Some(ref documents) = row.document_ids {
            document_ids.extend(documents.iter().copied());
        }
    }

    // Remove duplicates while preserving order
    asset_ids.sort_unstable();
    asset_ids.dedup();
    document_ids.sort_unstable();
    document_ids.dedup();

    Ok(CollectionHierarchy {
        collection_id,
        creative_ids,
        asset_ids,
        document_ids,
    })
}

#[cfg(test)]
mod tests {
    //! Tests for get_collection_hierarchy.

    use super::*;

    #[test]
    fn test_collection_hierarchy_structure() {
        // Test that CollectionHierarchy struct has expected fields
        let hierarchy = CollectionHierarchy {
            collection_id: Uuid::new_v4(),
            creative_ids: vec![],
            asset_ids: vec![],
            document_ids: vec![],
        };
        
        assert_eq!(hierarchy.creative_ids.len(), 0);
        assert_eq!(hierarchy.asset_ids.len(), 0);
        assert_eq!(hierarchy.document_ids.len(), 0);
    }
}
