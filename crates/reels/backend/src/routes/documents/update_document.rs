//! Handler for updating an existing document entry's content.
//!
//! This module provides the main HTTP handler for document updates, coordinating
//! permission checks, validation, and database operations. It implements clean
//! architecture principles with minimal transaction scope for optimal performance.
//! Admin users can update any document regardless of ownership or shares.

// Using error types and parameter structs from separate modules per coding guidelines

#[utoipa::path(
    put,
    path = "/api/documents/{id}",
    tag = "Documents",
    request_body = crate::routes::documents::update_document_request::UpdateDocumentRequest,
    responses(
        (status = 200, description = "Document updated", body = crate::routes::documents::responses::DocumentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only administrators can make documents public, or user lacks edit permissions", body = crate::routes::error_response::ErrorResponse),
        (status = 422, description = "Validation Error", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse),
    ),
    security(("user_auth" = []))
)]
#[actix_web::put("/{id}")]
pub async fn update_document(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    req: actix_web::web::Json<crate::routes::documents::update_document_request::UpdateDocumentRequest>,
) -> impl actix_web::Responder {
    let document_id = path.into_inner();

    // Step 1: Validate request
    if let std::result::Result::Err(e) = validate_update_request(&req) {
        log::warn!(
            "Validation failed for update document request by user {}: {:?}",
            claims.user_id,
            e
        );
        return crate::routes::documents::update_document_error::UpdateDocumentError::to_http_response(e);
    }

    // Step 2: Check permissions
    let permission_result = match check_document_permissions(&pool, document_id, claims.user_id, claims.is_admin).await {
        std::result::Result::Ok(result) => result,
        std::result::Result::Err(e) => return crate::routes::documents::update_document_error::UpdateDocumentError::to_http_response(e),
    };

    // Step 3: Validate admin permissions
    if let std::result::Result::Err(e) = validate_public_permission(&req, &claims) {
        return crate::routes::documents::update_document_error::UpdateDocumentError::to_http_response(e);
    }

    // Step 4: Perform update
    let params = crate::routes::documents::document_update_params::DocumentUpdateParams::from_request(&req, claims.user_id);
    let updated_document = match update_document_in_db(&pool, document_id, &params).await {
        std::result::Result::Ok(doc) => doc,
        std::result::Result::Err(e) => return crate::routes::documents::update_document_error::UpdateDocumentError::to_http_response(e),
    };

    // Step 5: Return response
    let response = crate::routes::documents::responses::DocumentResponse {
        document: updated_document,
        creator_email: permission_result.creator_email,
        current_user_access_level: permission_result.effective_access_level,
    };

    actix_web::HttpResponse::Ok().json(response)
}

/// Pure function - no side effects, validates request
fn validate_update_request(req: &crate::routes::documents::update_document_request::UpdateDocumentRequest) -> std::result::Result<(), crate::routes::documents::update_document_error::UpdateDocumentError> {
    validator::Validate::validate(req).map_err(|e| {
        crate::routes::documents::update_document_error::UpdateDocumentError::ValidationError(std::format!("Validation failed: {e}"))
    })
}

/// Pure function - validate admin permissions for public documents
fn validate_public_permission(
    req: &crate::routes::documents::update_document_request::UpdateDocumentRequest,
    claims: &crate::auth::tokens::Claims,
) -> std::result::Result<(), crate::routes::documents::update_document_error::UpdateDocumentError> {
    crate::queries::documents::check_update_permissions::validate_admin_permission_for_public(req.is_public, claims.is_admin)
        .map_err(|msg| {
            log::warn!(
                "Non-admin user {} attempted to make document public",
                claims.user_id
            );
            crate::routes::documents::update_document_error::UpdateDocumentError::Forbidden(msg)
        })
}

/// Read-only queries, no transaction needed - checks document permissions
async fn check_document_permissions(
    pool: &sqlx::PgPool,  
    document_id: uuid::Uuid,
    authenticated_user_id: uuid::Uuid,
    is_admin: bool,
) -> std::result::Result<crate::queries::documents::check_update_permissions::DocumentPermissionResult, crate::routes::documents::update_document_error::UpdateDocumentError> {
    crate::queries::documents::check_update_permissions::check_update_permissions(pool, document_id, authenticated_user_id, is_admin)
        .await
        .map_err(|e| {
            log::error!("Permission check: Error checking document permissions for doc {document_id}: {e}");
            match e {
                sqlx::Error::RowNotFound => crate::routes::documents::update_document_error::UpdateDocumentError::NotFound("Document not found".to_string()),
                _ => crate::routes::documents::update_document_error::UpdateDocumentError::DatabaseError("Permission check failed".to_string()),
            }
        })
}

/// Focused update function with minimal transaction scope
async fn update_document_in_db(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
    params: &crate::routes::documents::document_update_params::DocumentUpdateParams,
) -> std::result::Result<crate::db::documents::Document, crate::routes::documents::update_document_error::UpdateDocumentError> {
    // TRANSACTION STARTS HERE - Only for the actual update!
    let mut tx = pool.begin().await.map_err(|e| {
        log::error!("Failed to begin transaction: {e}");
        crate::routes::documents::update_document_error::UpdateDocumentError::DatabaseError("Failed to start database transaction".to_string())
    })?;

    let updated_document = crate::queries::documents::update_document_entry::update_document_entry_with_visibility(
        &mut tx,
        document_id,
        params.title.as_deref(),
        params.content.as_deref(),
        params.is_task,
        params.include_research.clone(),
        params.is_public,
        params.user_id,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to update document {document_id}: {e}");
        crate::routes::documents::update_document_error::UpdateDocumentError::DatabaseError("Failed to update document".to_string())
    })?;

    tx.commit().await.map_err(|e| {
        log::error!("Failed to commit transaction for document update {document_id}: {e}");
        crate::routes::documents::update_document_error::UpdateDocumentError::DatabaseError("Failed to finalize document update".to_string())
    })?;
    // TRANSACTION ENDS HERE - Automatic rollback via RAII if any error above

    std::result::Result::Ok(updated_document)
}

#[cfg(test)]
mod tests {
    //! Unit tests for document update handler and validation functions.
    
    #[test]
    fn test_validate_update_request_valid() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::Some("Test Title".to_string()),
            content: std::option::Option::Some("Test Content".to_string()),
            is_task: std::option::Option::Some(false),
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(false),
        };
        
        let result = super::validate_update_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_permission_admin_allowed() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::None,
            content: std::option::Option::None,
            is_task: std::option::Option::None,
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(true),
        };
        
        let claims = crate::auth::tokens::Claims {
            user_id: uuid::Uuid::new_v4(),
            is_admin: true,
            email: "admin@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: 0,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };
        
        let result = super::validate_public_permission(&req, &claims);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_permission_non_admin_rejected() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::None,
            content: std::option::Option::None,
            is_task: std::option::Option::None,
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(true),
        };
        
        let claims = crate::auth::tokens::Claims {
            user_id: uuid::Uuid::new_v4(),
            is_admin: false,
            email: "user@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: 0,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };
        
        let result = super::validate_public_permission(&req, &claims);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_public_permission_private_document_allowed() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::None,
            content: std::option::Option::None,
            is_task: std::option::Option::None,
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(false),
        };
        
        let claims = crate::auth::tokens::Claims {
            user_id: uuid::Uuid::new_v4(),
            is_admin: false,
            email: "user@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: 0,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };
        
        let result = super::validate_public_permission(&req, &claims);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_permission_no_visibility_change_allowed() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::Some("Test Title".to_string()),
            content: std::option::Option::None,
            is_task: std::option::Option::None,
            include_research: std::option::Option::None,
            is_public: std::option::Option::None,
        };
        
        let claims = crate::auth::tokens::Claims {
            user_id: uuid::Uuid::new_v4(),
            is_admin: false,
            email: "user@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::None,
            exp: 0,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::None,
        };
        
        let result = super::validate_public_permission(&req, &claims);
        assert!(result.is_ok());
    }
}
