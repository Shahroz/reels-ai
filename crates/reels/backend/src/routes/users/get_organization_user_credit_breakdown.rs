//! Route handler to get per-user credit breakdown for an organization.
//!
//! This endpoint returns credit usage statistics broken down by user
//! within a specific organization. It can optionally filter to specific users.
//! Requires the requesting user to be a member of the organization.

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct OrganizationUserBreakdownParams {
    #[param(value_type = String, format = "uuid")]
    pub organization_id: uuid::Uuid,
    #[param(example = "2024-01-01")]
    pub start_date: String,
    #[param(example = "2024-12-31")]
    pub end_date: String,
    #[param(example = "uuid1,uuid2,uuid3")]
    #[serde(default)]
    pub user_ids: Option<String>, // Comma-separated UUIDs
}

#[utoipa::path(
    get,
    path = "/api/users/organization-credit-breakdown",
    tag = "Users",
    params(
        OrganizationUserBreakdownParams
    ),
    responses(
        (status = 200, description = "Per-user credit breakdown", body = Vec<crate::queries::credit_transactions::get_organization_user_breakdown::UserCreditUsageSummary>),
        (status = 400, description = "Invalid parameters", body = crate::routes::error_response::ErrorResponse),
        (status = 403, description = "User is not a member of this organization", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse),
    ),
    security(
        ("bearer" = [])
    )
)]
#[actix_web::get("/organization-credit-breakdown")]
pub async fn get_organization_user_credit_breakdown_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<OrganizationUserBreakdownParams>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let organization_id = params.organization_id;
    
    log::info!(
        "User {} requesting per-user credit breakdown for organization {} from {} to {}",
        user_id,
        organization_id,
        params.start_date,
        params.end_date
    );
    
    // Verify user is a member of the organization
    match crate::queries::organizations::verify_organization_membership(pool.get_ref(), user_id, organization_id).await {
        Ok(true) => {
            log::info!("User {} is a member of organization {}", user_id, organization_id);
        },
        Ok(false) => {
            log::warn!(
                "User {} attempted to access credit breakdown for organization {} but is not a member",
                user_id,
                organization_id
            );
            return actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse {
                error: "You are not a member of this organization".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to verify organization membership: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to verify organization membership".to_string(),
            });
        }
    }
    
    // Parse optional user_ids filter
    let user_ids = if let Some(ids_str) = &params.user_ids {
        let parsed: Result<Vec<uuid::Uuid>, _> = ids_str
            .split(',')
            .map(|s| uuid::Uuid::parse_str(s.trim()))
            .collect();
        
        match parsed {
            Ok(ids) => {
                log::info!("Filtering to {} specific users", ids.len());
                Some(ids)
            },
            Err(e) => {
                log::warn!("Invalid user_ids format: {}", e);
                return actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                    error: format!("Invalid user_ids format: {}", e),
                });
            }
        }
    } else {
        None
    };
    
    // Fetch breakdown
    match crate::queries::credit_transactions::get_organization_user_breakdown::get_organization_user_breakdown(
        pool.get_ref(),
        organization_id,
        &params.start_date,
        &params.end_date,
        user_ids,
    ).await {
        Ok(breakdown) => {
            log::info!(
                "Retrieved {} user credit summaries for organization {}",
                breakdown.len(),
                organization_id
            );
            actix_web::HttpResponse::Ok().json(breakdown)
        },
        Err(e) => {
            log::error!("Failed to fetch organization user breakdown: {}", e);
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to fetch credit breakdown".to_string(),
            })
        }
    }
}

