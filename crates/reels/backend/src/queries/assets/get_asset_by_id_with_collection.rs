//! Defines the `get_asset_by_id_with_collection` database query function.
//!
//! This function retrieves a single asset from the `assets` table by its ID,
//! including collection details if the asset belongs to a collection.
//! Adheres to the project's Rust coding standards.


pub async fn get_asset_by_id_with_collection(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<crate::routes::assets::responses::AssetWithCollection>, sqlx::Error> {
    // Fetch user's organization IDs for permission checking
    let org_ids: Vec<uuid::Uuid> = sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_all(pool)
    .await?;

    // Query asset with collection details and organization shares using LEFT JOIN
    let asset_result = sqlx::query!(
        r#"
        SELECT 
            a.id, a.user_id, a.name, a.type, a.gcs_object_name, a.url, 
            a.collection_id, a.metadata, a.created_at, a.updated_at, a.is_public,
            c.id as "collection_id_result?", c.user_id as "collection_user_id?", 
            c.organization_id as "collection_organization_id?",
            c.name as "collection_name?", c.metadata as "collection_metadata?",
            c.created_at as "collection_created_at?", c.updated_at as "collection_updated_at?",
            COALESCE(
                CASE WHEN a.user_id = $2 THEN 'owner' END,
                os_user.access_level::TEXT,
                os_org.access_level::TEXT,
                cs_user.access_level::TEXT,
                cs_org.access_level::TEXT
            ) as "current_user_access_level?"
        FROM assets a
        LEFT JOIN collections c ON a.collection_id = c.id
        LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
            AND os_user.object_type = 'asset' 
            AND os_user.entity_type = 'user' 
            AND os_user.entity_id = $2
        LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
            AND os_org.object_type = 'asset' 
            AND os_org.entity_type = 'organization' 
            AND os_org.entity_id = ANY($3)
        LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
            AND cs_user.object_type = 'collection' 
            AND cs_user.entity_type = 'user' 
            AND cs_user.entity_id = $2
        LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
            AND cs_org.object_type = 'collection' 
            AND cs_org.entity_type = 'organization' 
            AND cs_org.entity_id = ANY($3)
        WHERE a.id = $1 
            AND (a.user_id = $2 OR a.is_public OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
        "#,
        asset_id,
        user_id,
        &org_ids[..]
    )
    .fetch_optional(pool)
    .await?;

    match asset_result {
        Some(row) => {
            let collection = if let Some(collection_id) = row.collection_id_result {
                Some(crate::db::collections::Collection {
                    id: collection_id,
                    user_id: row.collection_user_id.unwrap(),
                    organization_id: row.collection_organization_id,
                    name: row.collection_name.unwrap(),
                    metadata: row.collection_metadata,
                    created_at: row.collection_created_at.unwrap(),
                    updated_at: row.collection_updated_at.unwrap(),
                })
            } else {
                None
            };

            std::result::Result::Ok(std::option::Option::Some(crate::routes::assets::responses::AssetWithCollection {
                id: row.id,
                user_id: row.user_id,
                name: row.name,
                r#type: row.r#type,
                gcs_object_name: row.gcs_object_name,
                url: row.url,
                collection_id: row.collection_id,
                metadata: row.metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                is_public: row.is_public,
                current_user_access_level: row.current_user_access_level,
                collection,
            }))
        }
        None => std::result::Result::Ok(std::option::Option::None),
    }
} 