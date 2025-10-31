//! Checks document update permissions for authenticated users using pool connections.
//!
//! This function performs comprehensive permission checking for document updates,
//! including ownership verification, share-based access, and admin permission validation
//! for public document creation. It uses pool connections for read-only operations
//! before any transactions are started, optimizing connection usage and performance.
//! Admin users can update any document regardless of ownership or shares.

/// Minimal permission check result containing only necessary access information.
#[derive(Debug)]
pub struct DocumentPermissionResult {
    pub owner_user_id: Option<uuid::Uuid>,
    pub creator_email: Option<std::string::String>,
    pub effective_access_level: Option<std::string::String>,
}

/// Validates admin permissions for public document operations.
///
/// This function checks if the user attempting to make a document public has
/// administrator privileges. Only admins can create or modify public documents.
pub fn validate_admin_permission_for_public(
    is_public_requested: Option<bool>,
    is_admin: bool,
) -> std::result::Result<(), std::string::String> {
    if let Some(true) = is_public_requested {
        if !is_admin {
            return std::result::Result::Err(
                "Only administrators can make documents public".to_string()
            );
        }
    }
    std::result::Result::Ok(())
}

/// Checks document update permissions using pool connections.
///
/// This function verifies that the authenticated user has permission to update
/// the specified document. It checks document ownership and any editor-level
/// shares granted to the user or their organizations. Admin users can update
/// any document regardless of ownership or shares. Returns permission details
/// needed for the update operation.
pub async fn check_update_permissions(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
    authenticated_user_id: uuid::Uuid,
    is_admin: bool,
) -> std::result::Result<DocumentPermissionResult, sqlx::Error> {
    // Admin users can update any document
    if is_admin {
        let admin_access_details = sqlx::query!(
            r#"
            SELECT
                d.user_id AS owner_user_id,
                (SELECT email FROM users WHERE id = d.user_id) AS creator_email
            FROM documents d
            WHERE d.id = $1
            "#,
            document_id
        )
        .fetch_optional(pool)
        .await?;

        let admin_access_details = admin_access_details.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        return std::result::Result::Ok(DocumentPermissionResult {
            owner_user_id: admin_access_details.owner_user_id,
            creator_email: admin_access_details.creator_email,
            effective_access_level: Some("admin".to_string()),
        });
    }

    // Fetch organization memberships using our new pool-based function
    let org_ids = crate::queries::documents::fetch_user_organization_ids_from_pool::fetch_user_organization_ids_from_pool(
        pool,
        authenticated_user_id,
    ).await?;

    // Check document access including both direct document shares and collection-level shares
    let access_details = sqlx::query!(
        r#"
        WITH DocumentShares AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'document' AND os.object_id = $1
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $2)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                )
        ),
        CollectionShares AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            INNER JOIN documents d ON d.collection_id = os.object_id
            WHERE d.id = $1 AND os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $2)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                )
        ),
        BestDocumentShare AS (
            SELECT access_level FROM DocumentShares WHERE rn = 1
        ),
        BestCollectionShare AS (
            SELECT access_level FROM CollectionShares WHERE rn = 1
        )
        SELECT
            d.user_id AS owner_user_id,
            (SELECT email FROM users WHERE id = d.user_id) AS creator_email,
            COALESCE(
                (SELECT access_level::TEXT FROM BestDocumentShare),
                (SELECT access_level::TEXT FROM BestCollectionShare)
            ) AS shared_access_level
        FROM documents d
        WHERE d.id = $1
        "#,
        document_id,            // $1
        authenticated_user_id,  // $2
        &org_ids                // $3: &Vec<Uuid> for ANY($N::UUID[])
    )
    .fetch_optional(pool)
    .await?;

    let access_details = access_details.ok_or_else(|| {
        sqlx::Error::RowNotFound
    })?;

    let effective_access_level = if Some(authenticated_user_id) == access_details.owner_user_id {
        Some("owner".to_string())
    } else {
        access_details.shared_access_level.clone()
    };

    // Check if user has editor or owner permissions
    if !(effective_access_level.as_deref() == Some("owner") || effective_access_level.as_deref() == Some("editor")) {
        return std::result::Result::Err(sqlx::Error::RowNotFound);
    }

    std::result::Result::Ok(DocumentPermissionResult {
        owner_user_id: access_details.owner_user_id,
        creator_email: access_details.creator_email,
        effective_access_level,
    })
}

#[cfg(test)]
mod tests {
    //! Tests for check_update_permissions.
    //!
    //! These are conceptual tests as they require a live database connection and async runtime.
    //! They outline the logic that would be used in an integration testing environment.

    #[test]
    fn test_validate_admin_permission_allows_admin_public() {
        // Test that admin users can make documents public
        let result = super::validate_admin_permission_for_public(Some(true), true);
        assert!(result.is_ok(), "Admin should be allowed to make documents public");
    }

    #[test]
    fn test_validate_admin_permission_blocks_non_admin_public() {
        // Test that non-admin users cannot make documents public
        let result = super::validate_admin_permission_for_public(Some(true), false);
        assert!(result.is_err(), "Non-admin should not be allowed to make documents public");
        assert_eq!(
            result.unwrap_err(),
            "Only administrators can make documents public"
        );
    }

    #[test]
    fn test_validate_admin_permission_allows_private_documents() {
        // Test that any user can make documents private or leave them unchanged
        let result_private = super::validate_admin_permission_for_public(Some(false), false);
        assert!(result_private.is_ok(), "Any user should be allowed to make documents private");

        let result_unchanged = super::validate_admin_permission_for_public(None, false);
        assert!(result_unchanged.is_ok(), "Any user should be allowed to leave visibility unchanged");
    }

    // Note: check_update_permissions requires a database connection and is tested
    // thoroughly in the integration test suite with real data scenarios.
} 