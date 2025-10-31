//! Handler for updating a user's credits via admin endpoint.
//!
//! This endpoint allows administrators to set the credit balance for a user's personal
//! organization. It first retrieves the user's personal organization, then updates the
//! organization_credit_allocation with transaction logging, and finally creates an audit
//! log entry. Only admin users can access this endpoint.

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}/credits",
    tag = "Admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "The ID of the user whose credits to update")
    ),
    request_body = crate::routes::admin::users::update_user_credits_request::UpdateUserCreditsRequest,
    responses(
        (status = 200, description = "Successfully updated user credits", body = crate::routes::admin::users::update_user_credits_response::UpdateUserCreditsResponse),
        (status = 400, description = "Invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "User or personal organization not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::put("/{user_id}/credits")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn update_user_credits_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    user_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::users::update_user_credits_request::UpdateUserCreditsRequest>,
) -> impl actix_web::Responder {
    let user_id = user_id.into_inner();
    let new_credits = bigdecimal::BigDecimal::from(payload.credits);
    
    // Get the user's personal organization
    let personal_org = match crate::queries::organizations::get_user_personal_organization::get_user_personal_organization(
        pool.get_ref(),
        user_id,
    )
    .await
    {
        Ok(Some(org)) => org,
        Ok(None) => {
            log::warn!("User {} does not have a personal organization", user_id);
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: "User does not have a personal organization".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to get user personal organization: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve user organization.".to_string(),
                },
            );
        }
    };
    
    // Update the organization credits with transaction logging
    let updated_allocation = match crate::queries::organization_credit_allocation::admin_update_organization_credits_with_transaction::admin_update_organization_credits_with_transaction(
        pool.get_ref(),
        personal_org.id,
        new_credits.clone(),
        auth_claims.user_id,
    )
    .await
    {
        Ok(allocation) => allocation,
        Err(e) => {
            log::error!("Failed to update organization credits: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to update user credits.".to_string(),
                },
            );
        }
    };
    
    // Create audit log entry
    // ERROR HANDLING POLICY: We always fail the request if audit log creation fails
    // to ensure complete audit trail for compliance (Option A - data consistency over availability)
    let metadata = serde_json::json!({
        "user_id": user_id,
        "organization_id": personal_org.id,
        "new_credits": new_credits.to_string(),
        "organization_name": personal_org.name
    });
    
    if let Err(e) = crate::queries::audit_logs::create_audit_log::create_audit_log(
        pool.get_ref(),
        auth_claims.user_id,
        crate::db::audit_action::AuditAction::UpdateUserCredits,
        "User",
        Some(user_id),
        Some(metadata),
    )
    .await
    {
        log::error!("Failed to create audit log for user credits update: {}", e);
        // Always fail to ensure complete audit trail (Option A)
        // Note: Credit transaction was already committed in admin_update_organization_credits_with_transaction
        // This is a known limitation - the credit update succeeded but audit log failed
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: "Failed to create audit log. Credits were updated but audit trail is incomplete.".to_string(),
            },
        );
    }
    
    log::info!(
        "Admin {} updated user {} credits to {}",
        auth_claims.user_id,
        user_id,
        new_credits
    );
    
    actix_web::HttpResponse::Ok().json(
        crate::routes::admin::users::update_user_credits_response::UpdateUserCreditsResponse {
            user_id,
            organization_id: personal_org.id,
            credits_remaining: updated_allocation.credits_remaining,
            message: "User credits updated successfully".to_string(),
        },
    )
}

