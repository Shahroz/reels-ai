//! Defines the `count_assets` database query function.
//!
//! This function counts the number of assets belonging to a user,
//! with an optional search filter on the asset name and collection filter.
//! Adheres to the project's Rust coding standards.

pub async fn count_assets(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    collection_id: Option<uuid::Uuid>,
    is_public: Option<bool>,
    org_ids: &[uuid::Uuid],
    logo_related: Option<bool>,
) -> Result<Option<i64>, sqlx::Error> {
    // Handle empty org_ids case
    let org_ids_param = if org_ids.is_empty() {
        &[] as &[uuid::Uuid]
    } else {
        org_ids
    };

    // Use a unified query that includes organization-shared assets
    if let Some(coll_id) = collection_id {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT a.id)
            FROM assets a
            LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
                AND os_user.object_type = 'asset' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
                AND os_org.object_type = 'asset' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($2)
            LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
                AND cs_user.object_type = 'collection' 
                AND cs_user.entity_type = 'user' 
                AND cs_user.entity_id = $1
            LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
                AND cs_org.object_type = 'collection' 
                AND cs_org.entity_type = 'organization' 
                AND cs_org.entity_id = ANY($2)
            LEFT JOIN logo_collection_assets lca ON a.id = lca.asset_id
            WHERE a.name ILIKE $3
            AND a.collection_id = $4
            AND (
                CASE
                    WHEN $5::BOOLEAN IS NULL THEN
                        -- Default behavior: return user's assets, public assets, shared assets, and collection-shared assets
                        (a.user_id = $1 OR a.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                    WHEN $5::BOOLEAN = true THEN
                        -- Only public assets regardless of user_id
                        a.is_public = true
                    WHEN $5::BOOLEAN = false THEN
                        -- User's assets + organization shared assets, but exclude public assets
                        (a.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                    ELSE
                        -- Fallback to default behavior
                        (a.user_id = $1 OR a.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                END
            )
            AND (
                CASE
                    WHEN $6::BOOLEAN IS NULL THEN
                        -- No logo relation filter
                        true
                    WHEN $6::BOOLEAN = true THEN
                        -- Only assets linked to logos
                        lca.id IS NOT NULL
                    WHEN $6::BOOLEAN = false THEN
                        -- Only assets not linked to logos
                        lca.id IS NULL
                    ELSE
                        -- Fallback to no filter
                        true
                END
            )
            "#,
            user_id,
            org_ids_param,
            search_pattern,
            coll_id,
            is_public,
            logo_related
        )
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT a.id)
            FROM assets a
            LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
                AND os_user.object_type = 'asset' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
                AND os_org.object_type = 'asset' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($2)
            LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
                AND cs_user.object_type = 'collection' 
                AND cs_user.entity_type = 'user' 
                AND cs_user.entity_id = $1
            LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
                AND cs_org.object_type = 'collection' 
                AND cs_org.entity_type = 'organization' 
                AND cs_org.entity_id = ANY($2)
            LEFT JOIN logo_collection_assets lca ON a.id = lca.asset_id
            WHERE a.name ILIKE $3
            AND (
                CASE
                    WHEN $4::BOOLEAN IS NULL THEN
                        -- Default behavior: return user's assets, public assets, shared assets, and collection-shared assets
                        (a.user_id = $1 OR a.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                    WHEN $4::BOOLEAN = true THEN
                        -- Only public assets regardless of user_id
                        a.is_public = true
                    WHEN $4::BOOLEAN = false THEN
                        -- User's assets + organization shared assets, but exclude public assets
                        (a.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                    ELSE
                        -- Fallback to default behavior
                        (a.user_id = $1 OR a.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
                END
            )
            AND (
                CASE
                    WHEN $5::BOOLEAN IS NULL THEN
                        -- No logo relation filter
                        true
                    WHEN $5::BOOLEAN = true THEN
                        -- Only assets linked to logos
                        lca.id IS NOT NULL
                    WHEN $5::BOOLEAN = false THEN
                        -- Only assets not linked to logos
                        lca.id IS NULL
                    ELSE
                        -- Fallback to no filter
                        true
                END
            )
            "#,
            user_id,
            org_ids_param,
            search_pattern,
            is_public,
            logo_related
        )
        .fetch_one(pool)
        .await
    }
}