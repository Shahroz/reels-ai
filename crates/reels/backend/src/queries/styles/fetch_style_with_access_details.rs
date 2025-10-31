//! Fetches a style record with complete access and creator details.
//!
//! This function retrieves a style with comprehensive access level calculation
//! including share-based permissions, organization membership, and creator
//! information. Used after updates to return fresh data with proper access context.

/// User context for access level calculation
#[derive(Debug)]
pub struct UserAccessContext {
    pub user_id: uuid::Uuid,
    pub org_ids: std::vec::Vec<uuid::Uuid>,
}

/// Fetches a style with complete access details and creator information
/// 
/// Executes complex query with ShareAccess CTE to calculate user's access level
/// to the style including ownership, direct shares, and organization-based shares.
/// Returns complete StyleResponse with all context information.
#[tracing::instrument(skip(tx))]
pub async fn fetch_style_with_access_details(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    style_id: uuid::Uuid,
    user_context: UserAccessContext,
) -> std::result::Result<crate::routes::styles::responses::StyleResponse, sqlx::Error> {
    #[derive(sqlx::FromRow, Debug)]
    struct StyleWithAccessDetails {
        id: uuid::Uuid,
        user_id: std::option::Option<uuid::Uuid>,
        name: std::string::String,
        html_url: std::string::String,
        screenshot_url: std::string::String,
        is_public: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        creator_email: std::option::Option<std::string::String>,
        current_user_access_level: std::option::Option<std::string::String>,
    }

    let org_ids_slice = if user_context.org_ids.is_empty() { 
        &[] 
    } else { 
        &user_context.org_ids[..] 
    };

    let result = sqlx::query_as!(
        StyleWithAccessDetails,
        r#"
        WITH ShareAccess AS (
            SELECT object_id, MAX(access_level::text) as access_level
            FROM object_shares
            WHERE object_id = $3 AND object_type = 'style'
              AND ( 
                (entity_type = 'user' AND entity_id = $1) 
                OR 
                (entity_type = 'organization' AND entity_id = ANY($2)) 
              )
            GROUP BY object_id
        )
        SELECT 
            s.id as "id!", 
            s.user_id, 
            s.name as "name!", 
            s.html_url as "html_url!", 
            s.screenshot_url as "screenshot_url!", 
            s.is_public as "is_public!",
            s.created_at as "created_at!", 
            s.updated_at as "updated_at!", 
            u.email as "creator_email?",
            CASE 
                WHEN s.user_id = $1 THEN 'owner'
                ELSE sa.access_level
            END as "current_user_access_level?"
        FROM styles s
        LEFT JOIN users u ON s.user_id = u.id
        LEFT JOIN ShareAccess sa ON s.id = sa.object_id
        WHERE s.id = $3
        "#,
        user_context.user_id,
        org_ids_slice,
        style_id
    )
    .fetch_one(&mut **tx)
    .await?;

    std::result::Result::Ok(crate::routes::styles::responses::StyleResponse {
        style: crate::db::styles::Style {
            id: result.id,
            user_id: result.user_id,
            name: result.name,
            html_url: result.html_url,
            screenshot_url: result.screenshot_url,
            is_public: result.is_public,
            created_at: result.created_at,
            updated_at: result.updated_at,
        },
        creator_email: result.creator_email,
        current_user_access_level: result.current_user_access_level,
    })
}

/// Helper to create user access context from organization memberships
pub fn create_user_access_context(
    user_id: uuid::Uuid,
    org_memberships: std::vec::Vec<crate::db::organization_members::OrganizationMember>,
) -> UserAccessContext {
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships
        .into_iter()
        .map(|m| m.organization_id)
        .collect();

    UserAccessContext {
        user_id,
        org_ids,
    }
}

/// Helper to create minimal user access context (no organizations)
pub fn create_minimal_user_access_context(user_id: uuid::Uuid) -> UserAccessContext {
    UserAccessContext {
        user_id,
        org_ids: std::vec::Vec::new(),
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_user_access_context_creation() {
        // Test creating user access context
        let user_id = uuid::Uuid::new_v4();
        let context = super::create_minimal_user_access_context(user_id);
        
        assert_eq!(context.user_id, user_id);
        assert!(context.org_ids.is_empty());
    }

    #[test]
    fn test_user_access_context_with_orgs() {
        // Test creating user access context with organizations
        let user_id = uuid::Uuid::new_v4();
        let org1 = uuid::Uuid::new_v4();
        let org2 = uuid::Uuid::new_v4();
        
        let context = super::UserAccessContext {
            user_id,
            org_ids: std::vec![org1, org2],
        };
        
        assert_eq!(context.user_id, user_id);
        assert_eq!(context.org_ids.len(), 2);
        assert!(context.org_ids.contains(&org1));
        assert!(context.org_ids.contains(&org2));
    }

    #[test]
    fn test_access_level_scenarios() {
        // Test different access level scenarios
        
        // Owner access
        let owner_id = uuid::Uuid::new_v4();
        let owner_context = super::create_minimal_user_access_context(owner_id);
        assert_eq!(owner_context.user_id, owner_id);
        
        // Shared access (would be determined by query)
        let user_id = uuid::Uuid::new_v4();
        let org_id = uuid::Uuid::new_v4();
        let shared_context = super::UserAccessContext {
            user_id,
            org_ids: std::vec![org_id],
        };
        assert_eq!(shared_context.user_id, user_id);
        assert!(shared_context.org_ids.contains(&org_id));
    }
} 