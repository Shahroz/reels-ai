//! Defines the query for fetching a collection with its associated assets.
//!
//! This function retrieves a collection by its ID along with all assets that belong to it.
//! Supports pagination and filtering of the assets within the collection.
//! Returns a combined result with collection details and associated assets with provenance info.
//! Adheres to the one-item-per-file and FQN guidelines.

use crate::db::assets::{Asset, AssetWithProvenance};
use crate::db::collections::Collection;

#[derive(serde::Serialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct CollectionWithAssets {
    #[serde(flatten)]
    pub collection: Collection,
    pub assets: std::vec::Vec<AssetWithProvenance>,
    pub total_assets: i64,
}

pub async fn get_collection_with_assets(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
    search_pattern: &str,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<Option<CollectionWithAssets>, sqlx::Error> {
    // First, verify the collection exists and the user has access (ownership or sharing)
    let collection = sqlx::query_as!(
        Collection,
        r#"
        SELECT DISTINCT c.id, c.user_id, c.organization_id, c.name, c.metadata, c.created_at, c.updated_at
        FROM collections c
        LEFT JOIN object_shares os ON c.id = os.object_id AND os.object_type = 'collection'
        WHERE c.id = $1 AND (
            c.user_id = $2 OR 
            os.entity_id = $2 OR 
            os.entity_id IN (
                SELECT organization_id FROM organization_members 
                WHERE user_id = $2 AND status = 'active'
            )
        )
        "#,
        collection_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    let collection = match collection {
        Some(c) => c,
        None => return Ok(None),
    };



    // Get total count of assets in this collection matching the search
    let count_query = r#"
        SELECT COUNT(*) as count
        FROM assets 
        WHERE collection_id = $1 AND name ILIKE $2
    "#;
    
    let total_assets: i64 = sqlx::query_scalar(count_query)
        .bind(collection_id)
        .bind(search_pattern)
        .fetch_one(pool)
        .await?;

    // Determine the ORDER BY clause
    let order_clause = match (sort_by, sort_order) {
        ("name", "asc") => "ORDER BY a.name ASC",
        ("name", "desc") => "ORDER BY a.name DESC",
        ("type", "asc") => "ORDER BY a.type ASC",
        ("type", "desc") => "ORDER BY a.type DESC",
        ("created_at", "asc") => "ORDER BY a.created_at ASC",
        ("updated_at", "asc") => "ORDER BY a.updated_at ASC",
        ("updated_at", "desc") => "ORDER BY a.updated_at DESC",
        (_, _) => "ORDER BY a.created_at DESC",
    };

    // Get the paginated and filtered assets with provenance information
    let query_str = format!(
        r#"
        SELECT 
            a.id, a.user_id, a.name, a.type, a.gcs_object_name, a.url, 
            a.collection_id, a.metadata, a.created_at, a.updated_at, a.is_public,
            CASE WHEN pe.target_id IS NOT NULL THEN true ELSE false END as is_enhanced
        FROM assets a
        LEFT JOIN provenance_edges pe ON a.id = pe.target_id AND pe.target_type = 'asset'
        WHERE a.collection_id = $1 AND a.name ILIKE $2
        {}
        LIMIT $3 OFFSET $4
        "#,
        order_clause
    );

    let rows = sqlx::query(&query_str)
        .bind(collection_id)
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let assets: Vec<AssetWithProvenance> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            AssetWithProvenance {
                asset: Asset {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    gcs_object_name: row.get("gcs_object_name"),
                    url: row.get("url"),
                    collection_id: row.get("collection_id"),
                    metadata: row.get("metadata"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    is_public: row.get("is_public"),
                },
                is_enhanced: row.get("is_enhanced"),
            }
        })
        .collect();

    Ok(Some(CollectionWithAssets {
        collection,
        assets,
        total_assets,
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_collection_with_assets_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}