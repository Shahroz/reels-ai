//! Service implementation for CreditsGuard middleware.
//!
//! This service handles credit checking for both user and organization contexts,
//! reading organization_id from the x-organization-id header when present.

use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse},
    web, Error, HttpMessage, HttpResponse,
};
use futures::future::LocalBoxFuture;
use sqlx::PgPool;
use bigdecimal::BigDecimal;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tracing::instrument;
use uuid::Uuid;

use crate::middleware::auth::AuthenticatedUser;

/// Service for CreditsGuard that handles credit checking
pub struct CreditsGuardService<S> {
    pub service: Arc<S>,
    pub required_credits: i32,
}

impl<S> CreditsGuardService<S> {
    /// Check if organization has sufficient credits and user has permission
    pub async fn check_organization_credits(
        pool: &web::Data<PgPool>,
        user_id: Uuid,
        org_id: Uuid,
        required_credits: i32,
    ) -> Result<(), HttpResponse<BoxBody>> {
        // First, verify user is a member of the organization
        match crate::queries::organizations::verify_organization_membership::verify_organization_membership(
            pool.get_ref(),
            user_id,
            org_id,
        ).await {
            Ok(true) => {
                tracing::info!("User {} authorized to spend credits for organization {}", user_id, org_id);
            }
            Ok(false) => {
                tracing::warn!("User {} attempted to spend credits for organization {} without membership", user_id, org_id);
                let response = HttpResponse::Forbidden()
                    .json(serde_json::json!({
                        "error": "Forbidden",
                        "message": "Not a member of this organization",
                        "code": "NOT_ORGANIZATION_MEMBER",
                    }));
                return Err(response);
            }
            Err(e) => {
                tracing::error!("Failed to verify organization membership: {}", e);
                let response = HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": "Internal Server Error",
                        "message": "Failed to verify organization membership",
                        "code": "DATABASE_ERROR",
                    }));
                return Err(response);
            }
        }

        // Check organization credits
        match crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id(
            pool.get_ref(),
            org_id,
        ).await {
            Ok(Some(org_allocation)) => {
                if org_allocation.credits_remaining < BigDecimal::from(required_credits) {
                    tracing::warn!(
                        "Organization {} has insufficient credits. Required: {}, Available: {}",
                        org_id,
                        required_credits,
                        org_allocation.credits_remaining
                    );
                    let response = HttpResponse::PaymentRequired()
                        .json(serde_json::json!({
                            "error": "Insufficient Credits",
                            "message": format!(
                                "You need {} credits but only have {} credits remaining",
                                required_credits,
                                org_allocation.credits_remaining
                            ),
                            "code": "INSUFFICIENT_CREDITS",
                            "required_credits": required_credits,
                            "available_credits": org_allocation.credits_remaining,
                        }));
                    return Err(response);
                }
                tracing::info!(
                    "Organization {} has sufficient credits. Required: {}, Available: {}",
                    org_id,
                    required_credits,
                    org_allocation.credits_remaining
                );
                Ok(())
            }
            Ok(None) => {
                tracing::warn!("No credit allocation found for organization {}", org_id);
                let response = HttpResponse::PaymentRequired()
                    .json(serde_json::json!({
                        "error": "No Credits",
                        "message": "No credit allocation found for this organization",
                        "code": "NO_CREDITS",
                    }));
                Err(response)
            }
            Err(e) => {
                tracing::error!("Failed to get organization credit allocation: {:?}", e);
                let response = HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": "Internal Server Error",
                        "message": "Failed to check organization credits",
                        "code": "DATABASE_ERROR",
                    }));
                Err(response)
            }
        }
    }
}

/// Extract organization_id from request headers
/// 
/// This is a standalone function to extract organization_id from ServiceRequest headers.
/// It's used by both the CreditsGuardService and other middleware/components.
pub fn extract_organization_id_from_service_request(req: &ServiceRequest) -> Option<Uuid> {
    if let Some(header_value) = req.headers().get("x-organization-id") {
        if let Ok(org_id_str) = header_value.to_str() {
            uuid::Uuid::parse_str(org_id_str).ok()
        } else {
            None
        }
    } else {
        None
    }
}

impl<S> Service<ServiceRequest> for CreditsGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[instrument(skip(self, req))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let pool_data = req.app_data::<web::Data<PgPool>>().cloned();
        let srv = self.service.clone();
        let required_credits = self.required_credits;

        Box::pin(async move {
            // Get the authenticated user from the request extensions
            let user_id = req.extensions().get::<AuthenticatedUser>().map(|auth_user| {
                match auth_user {
                    AuthenticatedUser::Jwt(claims) => claims.user_id,
                    AuthenticatedUser::ApiKey(user_id) => *user_id,
                }
            });

            let user_id = if let Some(user_id) = user_id {
                user_id
            } else {
                // No authenticated user, return unauthorized
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Authentication required to check credits. Please login to continue.",
                        "code": "AUTHENTICATION_REQUIRED"
                    }));
                return Ok(req.into_response(response.map_into_boxed_body()));
            };

            // Try to get organization_id from custom header
            let organization_id = extract_organization_id_from_service_request(&req);

            // Get database pool
            let pool = match pool_data {
                Some(p) => p,
                None => {
                    let response = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal Server Error",
                            "message": "Connection not stable. Please try again later.",
                            "code": "CONNECTION_ERROR"
                        }));
                    return Ok(req.into_response(response.map_into_boxed_body()));
                }
            };

            // User credit context - check user credits
            match crate::queries::user_credit_allocation::is_old_user_exempt_from_credit_checks::is_old_user_exempt_from_credit_checks(pool.get_ref(), user_id).await {
                Ok(true) => {
                    // Old user exempt from credit checks, proceed with the request
                    tracing::info!("User {} is an old user exempt from credit checks, allowing request", user_id);
                    srv.call(req).await
                }
                Ok(false) => {
                    // Determine which credit context to use
                    if let Some(org_id) = organization_id {
                        // Organization credit context - check organization credits
                        match Self::check_organization_credits(&pool, user_id, org_id, required_credits).await {
                            Ok(()) => srv.call(req).await,
                            Err(response) => Ok(req.into_response(response.map_into_boxed_body())),
                        }
                    } else {
                        // Not an old user, proceed with normal credit checks
                        match crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool.get_ref(), user_id).await {
                            Ok(Some(credit_allocation)) => {
                                // Check if user has sufficient credits
                                if credit_allocation.credits_remaining >= BigDecimal::from(required_credits) {
                                    // User has sufficient credits, proceed with the request
                                    srv.call(req).await
                                } else {
                                    // Insufficient credits
                                    let response = HttpResponse::PaymentRequired()
                                        .json(serde_json::json!({
                                            "error": "Insufficient Credits",
                                            "message": format!(
                                                "You need {} credits but only have {} credits remaining",
                                                required_credits,
                                                credit_allocation.credits_remaining
                                            ),
                                            "code": "INSUFFICIENT_CREDITS",
                                            "required_credits": required_credits,
                                            "available_credits": credit_allocation.credits_remaining,
                                        }));
                                    Ok(req.into_response(response.map_into_boxed_body()))
                                }
                            }
                            Ok(None) => {
                                // No credit allocation found for user
                                let response = HttpResponse::PaymentRequired()
                                    .json(serde_json::json!({
                                        "error": "No Credits",
                                        "message": "No credit found for user. Please contact support.",
                                        "code": "NO_CREDITS",
                                    }));
                                Ok(req.into_response(response.map_into_boxed_body()))
                            }
                            Err(e) => {
                                // Database error
                                tracing::error!("Database error while checking user credits: {:?}", e);
                                let response = HttpResponse::InternalServerError()
                                    .json(serde_json::json!({
                                        "error": "Internal Server Error",
                                        "message": "Failed to check user credits",
                                        "code": "CONNECTION_ERROR"
                                    }));
                                Ok(req.into_response(response.map_into_boxed_body()))
                            }
                        } 
                    }
                }
                Err(e) => {
                    // Database error while checking old user status
                    tracing::error!("Database error while checking old user status: {:?}", e);
                    let response = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal Server Error",
                            "message": "Failed to check user status",
                            "code": "CONNECTION_ERROR"
                        }));
                    Ok(req.into_response(response.map_into_boxed_body()))
                }
            }
        })
    }
}

