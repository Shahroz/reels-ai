use actix_web::{post, web, HttpRequest, HttpResponse, Responder, HttpMessage};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;


use crate::middleware::auth::AuthenticatedUser;
use crate::services::billing::billing_factory::create_billing_service;
use crate::services::billing::billing_config::BillingConfig;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCustomerPortalSessionRequest {
    pub source: String,
    pub return_url: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateCustomerPortalSessionResponse {
    pub portal_url: String,
}

#[utoipa::path(
    post,
    path = "/api/billing/portal",
    tag = "Billing",
    request_body = CreateCustomerPortalSessionRequest,
    responses(
        (status = 200, description = "Customer portal session created successfully", body = CreateCustomerPortalSessionResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
#[post("/portal")]
#[instrument(skip(req))]
pub async fn create_customer_portal_session(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateCustomerPortalSessionRequest>,
) -> impl Responder {
    // Extract user ID from authenticated user
    let user_id = if let Some(auth_user) = req.extensions().get::<AuthenticatedUser>() {
        match auth_user {
            AuthenticatedUser::Jwt(claims) => claims.user_id,
            AuthenticatedUser::ApiKey(user_id) => *user_id,
        }
    } else {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        }));
    };

    let config = BillingConfig::from_env();
    match create_billing_service(&config) {
        Ok(billing_service) => {
            match billing_service.create_customer_portal_session(
                pool.get_ref(),
                user_id,
                &body.return_url,
            ).await {
                Ok(portal) => {
                    log::info!("Successfully created customer portal session for user {user_id}");
                    HttpResponse::Ok().json(CreateCustomerPortalSessionResponse {
                        portal_url: portal.portal_url,
                    })
                }
                Err(e) => {
                    let error_message = e.to_string();
                    log::error!("Failed to create customer portal session: {e}");
                    
                    // Handle validation errors as client errors (400)
                    if error_message.contains("Invalid return URL format") || 
                    error_message.contains("does not have a Stripe customer ID") {
                        HttpResponse::BadRequest().json(serde_json::json!({
                            "error": "Bad request",
                            "message": error_message
                        }))
                    } else {
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to create customer portal session",
                            "message": error_message
                        }))
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create billing service: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create billing service",
                "message": e
            }))
        }
    }
} 