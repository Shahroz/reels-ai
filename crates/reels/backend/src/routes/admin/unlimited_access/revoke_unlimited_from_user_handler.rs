//! Handler for revoking unlimited access from a user via admin endpoint.
//!
//! This endpoint allows administrators to revoke unlimited credit access from a user.
//! The grant is soft-deleted with revocation timestamp and reason recorded.
//! Creates an audit log entry for compliance. Only admin users can access this endpoint.

#[utoipa::path(
    delete,
    path = "/api/admin/unlimited-access/users/{user_id}/revoke",
    tag = "Admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "The ID of the user to revoke unlimited access")
    ),
    request_body = crate::routes::admin::unlimited_access::revoke_unlimited_from_user_request::RevokeUnlimitedFromUserRequest,
    responses(
        (status = 200, description = "Successfully revoked unlimited access", body = crate::routes::admin::unlimited_access::revoke_unlimited_from_user_response::RevokeUnlimitedFromUserResponse),
        (status = 400, description = "User does not have unlimited access", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "User or grant not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::delete("/users/{user_id}/revoke")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn revoke_unlimited_from_user_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    user_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::unlimited_access::revoke_unlimited_from_user_request::RevokeUnlimitedFromUserRequest>,
) -> impl actix_web::Responder {
    let user_id = user_id.into_inner();
    
    // Check if user has unlimited access (advisory check before transaction)
    match crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(
        pool.get_ref(),
        user_id,
    )
    .await
    {
        Ok(false) => {
            return actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User does not have unlimited access".to_string(),
                },
            );
        }
        Ok(true) => {}
        Err(e) => {
            log::error!("Failed to check unlimited access status: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to check unlimited access status.".to_string(),
                },
            );
        }
    }
    
    // Begin transaction to ensure atomic revoke + audit log creation
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to start revoke operation.".to_string(),
                },
            );
        }
    };
    
    // Revoke the unlimited access grant within transaction
    let grant = match sqlx::query_as!(
        crate::db::unlimited_access_grant::UnlimitedAccessGrant,
        r#"
        UPDATE unlimited_access_grants
        SET revoked_at = $1,
            revoked_by_user_id = $2,
            revoked_reason = $3,
            updated_at = $1
        WHERE user_id = $4
          AND revoked_at IS NULL
        RETURNING id, user_id, organization_id, granted_at, granted_by_user_id,
                  granted_reason, expires_at, revoked_at, revoked_by_user_id,
                  revoked_reason, notes, metadata, created_at, updated_at
        "#,
        chrono::Utc::now(),
        auth_claims.user_id,
        payload.reason,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(grant) => grant,
        Err(sqlx::Error::RowNotFound) => {
            // Race condition: grant was already revoked or expired between check and update
            return actix_web::HttpResponse::Conflict().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User no longer has active unlimited access grant".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to revoke unlimited access grant: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to revoke unlimited access.".to_string(),
                },
            );
        }
    };
    
    // Create audit log entry within same transaction
    let metadata = serde_json::json!({
        "user_id": user_id,
        "grant_id": grant.id,
        "reason": payload.reason
    });
    
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO audit_logs (admin_user_id, action_type, target_entity_type, target_entity_id, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        auth_claims.user_id,
        crate::db::audit_action::AuditAction::RevokeUnlimitedAccess.as_str(),
        "User",
        user_id,
        metadata
    )
    .execute(&mut *tx)
    .await
    {
        log::error!("Failed to create audit log for unlimited access revocation: {}", e);
        // Transaction will be rolled back automatically on drop
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to create audit log. Revocation not completed.".to_string(),
            },
        );
    }
    
    // Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit revoke transaction: {}", e);
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to finalize revoke operation.".to_string(),
            },
        );
    }
    
    log::info!(
        "Admin {} revoked unlimited access from user {}",
        auth_claims.user_id,
        user_id
    );
    
    actix_web::HttpResponse::Ok().json(
        crate::routes::admin::unlimited_access::revoke_unlimited_from_user_response::RevokeUnlimitedFromUserResponse {
            grant,
            message: "Unlimited access revoked successfully".to_string(),
        },
    )
}

