//! Checks if a user has permission to update a specific style.
//!
//! This function implements the business logic for style update permissions including
//! admin overrides for public styles, organization membership checking, and
//! current style state validation. Returns business-logic-aware permission results.

/// Parameters for style update permission check
#[derive(Debug)]
pub struct StyleUpdatePermissionParams {
    pub style_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub is_admin: bool,
}

/// Result of style permission check with context
#[derive(Debug)]
pub struct StyleUpdatePermissionResult {
    pub can_update: bool,
    pub is_public: bool,
    pub reason: std::string::String,
}

/// Checks if a user can update a specific style based on admin status and style state
/// 
/// Implements complex permission logic: admins can update any style, users can only
/// update their own private styles or styles shared with them. Public styles are
/// admin-only for updates.
#[tracing::instrument(skip(pool))]
pub async fn check_style_update_permissions(
    pool: &sqlx::PgPool,
    params: StyleUpdatePermissionParams,
) -> std::result::Result<StyleUpdatePermissionResult, actix_web::HttpResponse> {
    // First, check if the style exists and get its current state
    let style_info = match sqlx::query!(
        r#"
        SELECT user_id, is_public
        FROM styles 
        WHERE id = $1
        "#,
        params.style_id
    )
    .fetch_optional(pool)
    .await
    {
        std::result::Result::Ok(Some(style)) => style,
        std::result::Result::Ok(None) => {
            return std::result::Result::Err(actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Style not found."),
                }
            ));
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to fetch style info for permission check: {e}");
            return std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Database error during permission check."),
                }
            ));
        }
    };

    // Check admin override for public styles
    if style_info.is_public && !params.is_admin {
        return std::result::Result::Ok(StyleUpdatePermissionResult {
            can_update: false,
            is_public: true,
            reason: std::string::String::from("Only administrators can update public styles."),
        });
    }

    // For private styles or admin users, check ownership and shares
    let can_update = if params.is_admin {
        // Admins can update any style
        true
    } else if let Some(owner_id) = style_info.user_id {
        if owner_id == params.user_id {
            // User owns the style
            true
        } else {
            // Check if style is shared with user through organizations or direct shares
            match check_shared_access(pool, params.style_id, params.user_id).await {
                std::result::Result::Ok(has_access) => has_access,
                std::result::Result::Err(e) => {
                    log::error!("Failed to check shared access: {e}");
                    return std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::string::String::from("Failed to check shared access permissions."),
                        }
                    ));
                }
            }
        }
    } else {
        // Public style with no owner - admin only (already checked above)
        false
    };

    let reason = if can_update {
        if params.is_admin {
            std::string::String::from("Admin access granted.")
        } else if style_info.user_id == Some(params.user_id) {
            std::string::String::from("Owner access granted.")
        } else {
            std::string::String::from("Shared access granted.")
        }
    } else {
        std::string::String::from("Access denied: insufficient permissions.")
    };

    std::result::Result::Ok(StyleUpdatePermissionResult {
        can_update,
        is_public: style_info.is_public,
        reason,
    })
}

/// Helper function to check shared access through organizations and direct shares
async fn check_shared_access(
    pool: &sqlx::PgPool,
    style_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    // Get user's organization memberships
    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user::find_active_memberships_for_user(pool, user_id).await {
        std::result::Result::Ok(memberships) => memberships,
        std::result::Result::Err(e) => {
            log::error!("Failed to fetch organization memberships: {e}");
            return std::result::Result::Err(sqlx::Error::RowNotFound);
        }
    };
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice = if org_ids.is_empty() { &[] } else { &org_ids[..] };

    // Check for editor access through shares
    let has_access = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM object_shares
            WHERE object_id = $1 AND object_type = 'style' AND access_level = 'editor'
            AND (
                (entity_type = 'user' AND entity_id = $2)
                OR
                (entity_type = 'organization' AND entity_id = ANY($3))
            )
        )
        "#,
        style_id,
        user_id,
        org_ids_slice
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    std::result::Result::Ok(has_access)
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_admin_permission_params() {
        // Test admin permission parameters
        let params = super::StyleUpdatePermissionParams {
            style_id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            is_admin: true,
        };
        
        assert!(params.is_admin);
    }

    #[test]
    fn test_regular_user_params() {
        // Test regular user permission parameters
        let params = super::StyleUpdatePermissionParams {
            style_id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            is_admin: false,
        };
        
        assert!(!params.is_admin);
    }

    #[test]
    fn test_permission_result_structure() {
        // Test permission result construction
        let result = super::StyleUpdatePermissionResult {
            can_update: true,
            is_public: false,
            reason: std::string::String::from("Owner access granted."),
        };
        
        assert!(result.can_update);
        assert!(!result.is_public);
        assert_eq!(result.reason, "Owner access granted.");
    }
} 