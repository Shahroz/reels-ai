//! Lists collections with permission information for the authenticated user.
//!
//! This function returns collections accessible to a user along with their access level
//! (owner, editor, viewer). It combines owned collections and shared collections,
//! calculating the user's effective permission level for each.

use crate::routes::collections::responses::CollectionWithPermissions;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_collections_with_permissions(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: &str,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<CollectionWithPermissions>> {
    // First, get user's organization memberships for permission checking
    let org_memberships = crate::queries::organizations::find_active_memberships_for_user(pool, user_id)
        .await?;
    
    let org_ids: Vec<Uuid> = org_memberships.iter().map(|m| m.organization_id).collect();

    // We can't use dynamic queries with query_as!, so we use explicit match cases for type safety
    let collections = match (sort_by, sort_order) {
        ("name", "asc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.name ASC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        ("name", "desc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.name DESC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        ("created_at", "asc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.created_at ASC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        ("created_at", "desc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.created_at DESC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        ("updated_at", "asc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.updated_at ASC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        ("updated_at", "desc") => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.updated_at DESC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
        // Default fallback
        _ => sqlx::query_as!(
            CollectionWithPermissions,
            r#"
            SELECT DISTINCT 
                c.id, c.user_id, c.name, c.metadata, c.created_at, c.updated_at,
                COALESCE(
                    CASE WHEN c.user_id = $1 THEN 'owner' END,
                    os_user.access_level::TEXT,
                    os_org.access_level::TEXT
                ) as "current_user_access_level?"
            FROM collections c
            LEFT JOIN object_shares os_user ON c.id = os_user.object_id 
                AND os_user.object_type = 'collection' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $1
            LEFT JOIN object_shares os_org ON c.id = os_org.object_id 
                AND os_org.object_type = 'collection' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($5)
            WHERE (c.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) 
                AND c.name ILIKE $2
            ORDER BY c.created_at DESC LIMIT $3 OFFSET $4
            "#,
            user_id, search_pattern, limit, offset, &org_ids[..]
        ).fetch_all(pool).await?,
    };

    Ok(collections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collections_with_permissions_structure() {
        // Test that our CollectionWithPermissions struct is correctly defined
        let _collection = CollectionWithPermissions {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test Collection".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            current_user_access_level: Some("owner".to_string()),
        };
    }
}
