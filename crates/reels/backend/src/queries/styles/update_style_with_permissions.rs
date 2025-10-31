//! Updates a style record with permission-based access control.
//!
//! This function performs atomic style updates with complex permission checking
//! including organization-based access control and public/private visibility
//! transitions. Returns rows affected for validation purposes.

/// Parameters for updating a style with permission validation
#[derive(Debug)]
pub struct UpdateStyleWithPermissionsParams {
    pub style_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub org_ids: std::vec::Vec<uuid::Uuid>,
    pub name: std::string::String,
    pub html_url: std::string::String,
    pub screenshot_url: std::string::String,
    pub is_public: std::option::Option<bool>, // None means don't update visibility
    pub new_user_id: std::option::Option<std::option::Option<uuid::Uuid>>, // None means don't update ownership
}

/// Updates a style record with complex permission checking
/// 
/// Performs atomic UPDATE with permission-based WHERE clause including organization
/// membership checking. Handles public/private transitions and ownership changes.
/// Returns number of rows affected for permission validation.
#[tracing::instrument(skip(tx))]
pub async fn update_style_with_permissions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: UpdateStyleWithPermissionsParams,
) -> std::result::Result<u64, sqlx::Error> {
    let org_ids_slice = if params.org_ids.is_empty() { 
        &[] 
    } else { 
        &params.org_ids[..] 
    };

    let rows_affected = if let (Some(is_public_value), Some(user_id_value)) = (&params.is_public, &params.new_user_id) {
        // Update including visibility and ownership changes
        let result = sqlx::query!(
            r#"
                UPDATE styles 
                SET name = $1, html_url = $2, screenshot_url = $3, is_public = $7, user_id = $8, updated_at = NOW()
            WHERE id = $4 AND (
                user_id = $5 
                OR
                id IN (
                    SELECT object_id FROM object_shares
                    WHERE object_id = $4 AND object_type = 'style' AND access_level = 'editor'
                    AND (
                        (entity_type = 'user' AND entity_id = $5)
                        OR
                        (entity_type = 'organization' AND entity_id = ANY($6))
                    )
                )
            )
            "#,
            params.name,
            params.html_url,
            params.screenshot_url,
            params.style_id,
            params.user_id,
            org_ids_slice,
            is_public_value,
            user_id_value.clone()
        )
        .execute(&mut **tx)
        .await?;
        
        result.rows_affected()
    } else {
        // Update without visibility/ownership changes
        let result = sqlx::query!(
            r#"
                UPDATE styles 
                SET name = $1, html_url = $2, screenshot_url = $3, updated_at = NOW()
            WHERE id = $4 AND (
                user_id = $5 
                OR
                id IN (
                    SELECT object_id FROM object_shares
                    WHERE object_id = $4 AND object_type = 'style' AND access_level = 'editor'
                    AND (
                        (entity_type = 'user' AND entity_id = $5)
                        OR
                        (entity_type = 'organization' AND entity_id = ANY($6))
                    )
                )
            )
            "#,
            params.name,
            params.html_url,
            params.screenshot_url,
            params.style_id,
            params.user_id,
            org_ids_slice
        )
        .execute(&mut **tx)
        .await?;
        
        result.rows_affected()
    };

    std::result::Result::Ok(rows_affected)
}

/// Helper to create update parameters for visibility transitions
pub fn create_visibility_update_params(
    style_id: uuid::Uuid,
    user_id: uuid::Uuid,
    org_ids: std::vec::Vec<uuid::Uuid>,
    name: std::string::String,
    html_url: std::string::String,
    screenshot_url: std::string::String,
    requested_public: bool,
    current_user_id: uuid::Uuid,
) -> UpdateStyleWithPermissionsParams {
    let new_user_id = if requested_public { 
        // Public styles have no owner
        None 
    } else { 
        // Private styles are owned by current user
        Some(current_user_id) 
    };

    UpdateStyleWithPermissionsParams {
        style_id,
        user_id,
        org_ids,
        name,
        html_url,
        screenshot_url,
        is_public: Some(requested_public),
        new_user_id: Some(new_user_id),
    }
}

/// Helper to create update parameters without visibility changes
pub fn create_content_only_update_params(
    style_id: uuid::Uuid,
    user_id: uuid::Uuid,
    org_ids: std::vec::Vec<uuid::Uuid>,
    name: std::string::String,
    html_url: std::string::String,
    screenshot_url: std::string::String,
) -> UpdateStyleWithPermissionsParams {
    UpdateStyleWithPermissionsParams {
        style_id,
        user_id,
        org_ids,
        name,
        html_url,
        screenshot_url,
        is_public: None, // Don't update visibility
        new_user_id: None, // Don't update ownership
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_visibility_update_params_to_public() {
        // Test creating parameters for making a style public
        let style_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let params = super::create_visibility_update_params(
            style_id,
            user_id,
            std::vec::Vec::new(),
            std::string::String::from("Test Style"),
            std::string::String::from("https://example.com/style.html"),
            std::string::String::from("https://example.com/screenshot.png"),
            true, // Make public
            user_id,
        );
        
        assert_eq!(params.is_public, Some(true));
        assert_eq!(params.new_user_id, Some(None)); // Public styles have no owner
    }

    #[test]
    fn test_visibility_update_params_to_private() {
        // Test creating parameters for making a style private
        let style_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let params = super::create_visibility_update_params(
            style_id,
            user_id,
            std::vec::Vec::new(),
            std::string::String::from("Test Style"),
            std::string::String::from("https://example.com/style.html"),
            std::string::String::from("https://example.com/screenshot.png"),
            false, // Make private
            user_id,
        );
        
        assert_eq!(params.is_public, Some(false));
        assert_eq!(params.new_user_id, Some(Some(user_id))); // Private styles are owned
    }

    #[test]
    fn test_content_only_update_params() {
        // Test creating parameters for content-only updates
        let style_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let params = super::create_content_only_update_params(
            style_id,
            user_id,
            std::vec::Vec::new(),
            std::string::String::from("Updated Style"),
            std::string::String::from("https://example.com/updated.html"),
            std::string::String::from("https://example.com/updated.png"),
        );
        
        assert_eq!(params.is_public, None); // Don't change visibility
        assert_eq!(params.new_user_id, None); // Don't change ownership
        assert_eq!(params.name, "Updated Style");
    }
} 