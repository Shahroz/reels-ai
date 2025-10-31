//! Handler for updating a user's status via admin endpoint.
//!
//! This endpoint allows administrators to change a user's account status (active, inactive, etc).
//! The handler updates the user status in the database and creates an audit log entry for
//! compliance tracking. Only admin users can access this endpoint.

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}/status",
    tag = "Admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "The ID of the user to update")
    ),
    request_body = crate::routes::admin::users::update_user_status_request::UpdateUserStatusRequest,
    responses(
        (status = 200, description = "Successfully updated user status", body = crate::db::users::PublicUser),
        (status = 400, description = "Invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "User not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::put("/{user_id}/status")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn update_user_status_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    user_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::users::update_user_status_request::UpdateUserStatusRequest>,
) -> impl actix_web::Responder {
    let user_id = user_id.into_inner();
    let new_status = payload.status.to_string();
    
    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to update user status.".to_string(),
                },
            );
        }
    };
    
    // First, lock the user row to prevent concurrent updates
    // This prevents race conditions where concurrent status updates could create
    // inconsistent audit trails with incorrect "previous state" values
    let _current_user = match sqlx::query_as!(
        crate::db::users::User,
        r#"
        SELECT id, email, password_hash, stripe_customer_id, email_verified, is_admin, 
               status, feature_flags, created_at, updated_at, verification_token, 
               token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        FROM users
        WHERE id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User not found".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to lock user row: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to update user status.".to_string(),
                },
            );
        }
    };
    
    // Now update user status
    let updated_user = match sqlx::query_as!(
        crate::db::users::User,
        r#"
        UPDATE users
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, email, password_hash, stripe_customer_id, email_verified, is_admin, 
                  status, feature_flags, created_at, updated_at, verification_token, 
                  token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        "#,
        new_status.as_str(),
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User not found".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to update user status: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to update user status.".to_string(),
                },
            );
        }
    };
    
    // Create audit log entry
    // ERROR HANDLING POLICY: We always fail the request if audit log creation fails
    // to ensure complete audit trail for compliance (Option A - data consistency over availability)
    let metadata = serde_json::json!({
        "user_id": user_id,
        "new_status": new_status,
        "email": updated_user.email
    });
    
    if let Err(e) = crate::queries::audit_logs::create_audit_log::create_audit_log(
        &mut *tx,
        auth_claims.user_id,
        crate::db::audit_action::AuditAction::UpdateUserStatus,
        "User",
        Some(user_id),
        Some(metadata),
    )
    .await
    {
        log::error!("Failed to create audit log for user status update: {}", e);
        // Always fail to ensure complete audit trail (Option A)
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to create audit log. Operation rolled back.".to_string(),
            },
        );
    }
    
    // Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to update user status.".to_string(),
            },
        );
    }
    
    log::info!(
        "Admin {} updated user {} status to {}",
        auth_claims.user_id,
        user_id,
        new_status
    );
    
    actix_web::HttpResponse::Ok().json(crate::db::users::PublicUser::from(updated_user))
}

