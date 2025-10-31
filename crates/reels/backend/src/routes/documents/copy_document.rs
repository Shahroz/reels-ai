//! Handles the HTTP request for copying an existing document.
//!
//! This operation allows a user to duplicate a document. The copy will be
//! owned by the requesting user and marked as private (`is_public = false`).
//! It uses pool connections for read operations and minimal transaction scope
//! for write operations, following clean architecture principles for optimal performance.

#[utoipa::path(
    post,
    path = "/api/documents/{id}/copy",
    tag = "Documents",
    params(
        ("id" = uuid::Uuid, Path, description = "ID of the document to copy")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Document copied successfully", body = crate::routes::documents::responses::DocumentResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Original document not found or not accessible", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Failed to copy document", body = crate::routes::error_response::ErrorResponse)
    )
)]
#[actix_web::post("/{id}/copy")]
#[tracing::instrument(skip(pool, claims))]
pub async fn copy_document(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> actix_web::HttpResponse {
    let original_document_id = id.into_inner();
    let authenticated_user_id = claims.user_id;

    // Step 1: Fetch organization IDs
    let org_ids = match crate::queries::documents::fetch_user_organization_ids_from_pool::fetch_user_organization_ids_from_pool(
        &pool,
        authenticated_user_id,
    ).await {
        std::result::Result::Ok(ids) => ids,
        std::result::Result::Err(e) => {
            tracing::error!("Copy: Failed to fetch organization IDs for user {}: {}", authenticated_user_id, e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to fetch organization memberships for permission check".into(),
            });
        }
    };

    // Step 2: Fetch original document and access info
    let original_doc_info = match crate::queries::documents::fetch_document_for_copy_from_pool::fetch_document_for_copy_from_pool(
        &pool,
        original_document_id,
        authenticated_user_id,
        &org_ids,
    ).await {
        std::result::Result::Ok(std::option::Option::Some(info)) => info,
        std::result::Result::Ok(std::option::Option::None) => {
            return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: "Original document not found".into(),
            });
        }
        std::result::Result::Err(e) => {
            tracing::error!("Failed to fetch original document access details: {:?}", e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to retrieve original document details".into(),
            });
        }
    };

    // Step 3: Permission Check
    if original_doc_info.current_user_access_to_original.is_none() && !original_doc_info.is_public {
        return actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse {
            error: "You do not have permission to copy this document".into(),
        });
    }

    // Step 4: Fetch user email
    let new_creator_email = match crate::queries::documents::fetch_user_email_from_pool::fetch_user_email_from_pool(
        &pool,
        authenticated_user_id,
    ).await {
        std::result::Result::Ok(std::option::Option::Some(email)) => email,
        std::result::Result::Ok(std::option::Option::None) => { // Should not happen for an authenticated user
            tracing::error!("Authenticated user {} not found when fetching email for copy.", authenticated_user_id);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to retrieve copying user details".into(),
            });
        }
        std::result::Result::Err(e) => {
            tracing::error!("Error fetching copying user email for user {}: {}", authenticated_user_id, e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to retrieve copying user details".into(),
            });
        }
    };

    // TRANSACTION STARTS HERE - Only for the actual insert!
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to start database transaction".into(),
            });
        }
    };

    // Step 5: Insert the new record
    let created_record = match crate::queries::documents::insert_document_copy::insert_document_copy(
        &mut tx,
        authenticated_user_id,
        &original_doc_info,
    ).await {
        std::result::Result::Ok(record) => record,
        std::result::Result::Err(e) => {
            tracing::error!("Failed to insert new document copy: {:?}", e);
            if let std::result::Result::Err(rb_err) = tx.rollback().await { 
                tracing::error!("Rollback failed: {}", rb_err); 
            }
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to save document copy".into(),
            });
        }
    };

    // Step 6: Commit the transaction
    if let std::result::Result::Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction for document copy {}: {:?}", created_record.id, e);
        return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
            error: "Failed to finalize document copy".into(),
        });
    }
    // TRANSACTION ENDS HERE - Automatic rollback via RAII if any error above

    // Step 7: Construct the full Document response for the new copy
    let document = crate::db::documents::Document {
        id: created_record.id,
        user_id: created_record.user_id, // Should be Some(authenticated_user_id)
        title: created_record.title,
        content: created_record.content,
        sources: created_record.sources,
        status: created_record.status,
        created_at: created_record.created_at,
        updated_at: created_record.updated_at,
        is_public: created_record.is_public,
        is_task: created_record.is_task,
        include_research: created_record.include_research.map(crate::db::document_research_usage::DocumentResearchUsage::from),
        collection_id: std::option::Option::None, // New documents start without collection assignment
    };

    let response = crate::routes::documents::responses::DocumentResponse {
        document,
        creator_email: std::option::Option::Some(new_creator_email),
        current_user_access_level: std::option::Option::Some("owner".to_string()),
    };

    actix_web::HttpResponse::Created().json(response)
}
