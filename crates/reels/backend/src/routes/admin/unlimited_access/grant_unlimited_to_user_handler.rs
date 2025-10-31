//! Handler for granting unlimited access to a user via admin endpoint.
//!
//! This endpoint allows administrators to grant unlimited credit access to a specific user.
//! The grant can optionally have an expiration date. Creates an audit log entry for compliance.
//! Only admin users can access this endpoint.

#[utoipa::path(
    post,
    path = "/api/admin/unlimited-access/users/{user_id}/grant",
    tag = "Admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "The ID of the user to grant unlimited access")
    ),
    request_body = crate::routes::admin::unlimited_access::grant_unlimited_to_user_request::GrantUnlimitedToUserRequest,
    responses(
        (status = 200, description = "Successfully granted unlimited access", body = crate::routes::admin::unlimited_access::grant_unlimited_to_user_response::GrantUnlimitedToUserResponse),
        (status = 400, description = "Invalid input or user already has unlimited access", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "User not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/users/{user_id}/grant")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn grant_unlimited_to_user_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    user_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::unlimited_access::grant_unlimited_to_user_request::GrantUnlimitedToUserRequest>,
) -> impl actix_web::Responder {
    let user_id = user_id.into_inner();
    
    // Check if user already has unlimited access (advisory check before transaction)
    match crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(
        pool.get_ref(),
        user_id,
    )
    .await
    {
        Ok(true) => {
            return actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User already has unlimited access".to_string(),
                },
            );
        }
        Ok(false) => {}
        Err(e) => {
            log::error!("Failed to check unlimited access status: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to check unlimited access status.".to_string(),
                },
            );
        }
    }
    
    // Begin transaction to ensure atomic grant + audit log creation
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to start grant operation.".to_string(),
                },
            );
        }
    };
    
    // Create the unlimited access grant within transaction
    let grant = match sqlx::query_as!(
        crate::db::unlimited_access_grant::UnlimitedAccessGrant,
        r#"
        INSERT INTO unlimited_access_grants (
            user_id, granted_by_user_id, granted_reason, expires_at, notes
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, organization_id, granted_at, granted_by_user_id,
                  granted_reason, expires_at, revoked_at, revoked_by_user_id,
                  revoked_reason, notes, metadata, created_at, updated_at
        "#,
        user_id,
        auth_claims.user_id,
        payload.reason,
        payload.expires_at,
        payload.notes.as_deref()
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(grant) => grant,
        Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("idx_unlimited_access_grants_user_active") => {
            // Race condition: another admin granted access between our check and insert
            return actix_web::HttpResponse::Conflict().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User already has active unlimited access grant".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to create unlimited access grant: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to grant unlimited access.".to_string(),
                },
            );
        }
    };
    
    // Create audit log entry within same transaction
    let metadata = serde_json::json!({
        "user_id": user_id,
        "grant_id": grant.id,
        "reason": payload.reason,
        "expires_at": payload.expires_at,
        "notes": payload.notes
    });
    
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO audit_logs (admin_user_id, action_type, target_entity_type, target_entity_id, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        auth_claims.user_id,
        crate::db::audit_action::AuditAction::GrantUnlimitedAccess.as_str(),
        "User",
        user_id,
        metadata
    )
    .execute(&mut *tx)
    .await
    {
        log::error!("Failed to create audit log for unlimited access grant: {}", e);
        // Transaction will be rolled back automatically on drop
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to create audit log. Grant not created.".to_string(),
            },
        );
    }
    
    // Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit grant transaction: {}", e);
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to finalize grant operation.".to_string(),
            },
        );
    }
    
    log::info!(
        "Admin {} granted unlimited access to user {}",
        auth_claims.user_id,
        user_id
    );
    
    actix_web::HttpResponse::Ok().json(
        crate::routes::admin::unlimited_access::grant_unlimited_to_user_response::GrantUnlimitedToUserResponse {
            grant,
            message: "Unlimited access granted successfully".to_string(),
        },
    )
}

