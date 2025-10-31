//! Handler for updating organization credits via admin endpoint.
//!
//! This endpoint allows administrators to set the credit balance for an organization.
//! It updates the organization_credit_allocation with transaction logging and creates
//! an audit log entry. Only admin users can access this endpoint. Works for both team
//! and personal organizations.

#[utoipa::path(
    put,
    path = "/api/admin/organizations/{organization_id}/credits",
    tag = "Admin",
    params(
        ("organization_id" = uuid::Uuid, Path, description = "The ID of the organization whose credits to update")
    ),
    request_body = crate::routes::admin::organizations::update_organization_credits_request::UpdateOrganizationCreditsRequest,
    responses(
        (status = 200, description = "Successfully updated organization credits", body = crate::routes::admin::organizations::update_organization_credits_response::UpdateOrganizationCreditsResponse),
        (status = 400, description = "Invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::put("/{organization_id}/credits")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn update_organization_credits_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    organization_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<crate::routes::admin::organizations::update_organization_credits_request::UpdateOrganizationCreditsRequest>,
) -> impl actix_web::Responder {
    let organization_id = organization_id.into_inner();
    let new_credits = bigdecimal::BigDecimal::from(payload.credits);
    
    // Verify the organization exists
    let organization = match sqlx::query_as!(
        crate::db::organizations::Organization,
        r#"
        SELECT id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
        organization_id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(org)) => org,
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Organization not found".to_string(),
                },
            );
        }
        Err(e) => {
            log::error!("Failed to fetch organization: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve organization.".to_string(),
                },
            );
        }
    };
    
    // Update the organization credits with transaction logging
    let updated_allocation = match crate::queries::organization_credit_allocation::admin_update_organization_credits_with_transaction::admin_update_organization_credits_with_transaction(
        pool.get_ref(),
        organization_id,
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
                    error: "Failed to update organization credits.".to_string(),
                },
            );
        }
    };
    
    // Create audit log entry
    // ERROR HANDLING POLICY: We always fail the request if audit log creation fails
    // to ensure complete audit trail for compliance (Option A - data consistency over availability)
    let metadata = serde_json::json!({
        "organization_id": organization_id,
        "organization_name": organization.name,
        "new_credits": new_credits.to_string(),
        "is_personal": organization.is_personal
    });
    
    if let Err(e) = crate::queries::audit_logs::create_audit_log::create_audit_log(
        pool.get_ref(),
        auth_claims.user_id,
        crate::db::audit_action::AuditAction::UpdateOrganizationCredits,
        "Organization",
        Some(organization_id),
        Some(metadata),
    )
    .await
    {
        log::error!("Failed to create audit log for organization credits update: {}", e);
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
        "Admin {} updated organization {} credits to {}",
        auth_claims.user_id,
        organization_id,
        new_credits
    );
    
    actix_web::HttpResponse::Ok().json(
        crate::routes::admin::organizations::update_organization_credits_response::UpdateOrganizationCreditsResponse {
            organization_id,
            credits_remaining: updated_allocation.credits_remaining,
            message: "Organization credits updated successfully".to_string(),
        },
    )
}

