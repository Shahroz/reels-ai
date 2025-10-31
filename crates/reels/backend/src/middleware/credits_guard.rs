//! Credits consumption middleware for validating user credit availability.
//!
//! This middleware checks if a user has sufficient credits before allowing access to
//! credit-consuming endpoints. It extracts the user ID from the authenticated user
//! and validates credit availability against the user's credit allocation.
//!
//! ## Organization Credits
//!
//! **Important:** This middleware only checks USER credits, not organization credits.
//! For endpoints that support organization credit contexts (via optional `organization_id`
//! in request body), you should:
//!
//! 1. **Do NOT apply this middleware** to those endpoints
//! 2. **Handle credit checking in the route handler** using `credits_service::deduct_credits`
//! 3. **Follow the pattern** documented in `ORGANIZATION_CREDITS_ENDPOINT_PATTERN.md`
//!
//! This approach is cleaner because:
//! - Middleware runs before body parsing, making it hard to check `organization_id`
//! - Route handlers can properly authorize org membership before credit checks
//! - The `credits_service` provides a unified interface for both contexts
//!
//! ## Usage
//!
//! Apply this middleware ONLY to endpoints that exclusively use user credits:
//!
//! ```rust,ignore
//! use crate::middleware::credits_guard::RequireCredits;
//!
//! web::resource("/user-only-endpoint")
//!     .wrap(RequireCredits(10))  // Only checks user credits
//!     .route(web::post().to(handler))
//! ```
//!
//! For organization-aware endpoints, handle credits in the route handler instead:
//!
//! ```rust,ignore
//! // NO .wrap(RequireCredits()) here!
//! web::resource("/org-aware-endpoint")
//!     .route(web::post().to(handler))
//!
//! async fn handler(...) {
//!     // Determine context, verify auth, deduct credits via credits_service
//!     // See ORGANIZATION_CREDITS_ENDPOINT_PATTERN.md
//! }
//! ```

use crate::app_constants::credits_constants::{CreditOperation, CreditsConsumption};
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::credits_guard_service::CreditsGuardService;
use crate::queries::user_credit_allocation::{
    get_user_credit_allocation_by_user_id,
};
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use bigdecimal::BigDecimal;
use futures::future::{ok, LocalBoxFuture, Ready};
use futures::task::{Context, Poll};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::instrument;

/// Middleware to check if user has sufficient credits for the requested operation
/// Supports both user and organization credit contexts via x-organization-id header
#[derive(Clone)]
pub struct CreditsGuard {
    /// Number of credits required for this operation
    pub required_credits: i32,
}

impl CreditsGuard {
    /// Create a new credits guard with the specified credit requirement
    pub fn new(required_credits: i32) -> Self {
        Self { required_credits }
    }
}

impl<S> actix_web::dev::Transform<S, ServiceRequest> for CreditsGuard
where
    S: actix_web::dev::Service<
            ServiceRequest,
            Response = actix_web::dev::ServiceResponse<BoxBody>,
            Error = Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = actix_web::dev::ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = CreditsGuardService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CreditsGuardService {
            service: Arc::new(service),
            required_credits: self.required_credits,
        })
    }
}

/// Helper function to create a credits guard with the specified credit requirement
pub fn require_credits(credits: i32) -> CreditsGuard {
    CreditsGuard::new(credits)
}

/// Convenience functions for common credit-consuming operations

/// Create a credits guard for retouching images (1 credit per image)
pub fn require_retouch_images() -> CreditsGuard {
    CreditsGuard::new(CreditsConsumption::RETOUCH_IMAGES)
}

/// Dynamic credits guard for retouching images based on asset count
#[derive(Clone)]
pub struct DynamicRetouchImagesGuard;

impl DynamicRetouchImagesGuard {
    /// Create a new dynamic retouch images guard
    pub fn new() -> Self {
        Self
    }
}

impl Default for DynamicRetouchImagesGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Transform<S, ServiceRequest> for DynamicRetouchImagesGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = DynamicRetouchImagesGuardService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DynamicRetouchImagesGuardService {
            service: Arc::new(service),
        })
    }
}

pub struct DynamicRetouchImagesGuardService<S> {
    service: Arc<S>,
}

impl<S> Service<ServiceRequest> for DynamicRetouchImagesGuardService<S>
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
        let service = Arc::clone(&self.service);
        let pool = req.app_data::<web::Data<PgPool>>().cloned();

        Box::pin(async move {
            // Extract user ID from authenticated user
            let user_id = match req.extensions().get::<AuthenticatedUser>() {
                Some(AuthenticatedUser::Jwt(claims)) => Some(claims.user_id),
                Some(AuthenticatedUser::ApiKey(api_key)) => Some(*api_key),
                None => None,
            };

            let user_id = match user_id {
                Some(id) => id,
                None => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Authentication Required",
                        "message": "No valid authentication found",
                        "code": "AUTHENTICATION_REQUIRED",
                    }));
                    return Ok(req.into_response(response.map_into_boxed_body()));
                }
            };

            // Parse request body to get asset_ids and optional organization_id
            let (asset_count, organization_id) = match Self::extract_request_data(&req).await {
                Ok(data) => data,
                Err(response) => return Ok(req.into_response(response.map_into_boxed_body())),
            };

            // Calculate required credits (1 credit per asset)
            let required_credits = asset_count * CreditsConsumption::RETOUCH_IMAGES;

            // Get user's credit allocation
            let pool = match pool {
                Some(pool) => pool,
                None => {
                    let response = HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Database Error",
                        "message": "Database connection not available",
                        "code": "DATABASE_ERROR",
                    }));
                    return Ok(req.into_response(response.map_into_boxed_body()));
                }
            };

            // User credit context - check user credits (existing logic)
            match crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(pool.get_ref(), user_id).await {
                Ok(true) => {
                    // User has unlimited access grant, proceed with the request
                    tracing::info!(
                        "User {} has unlimited access grant, allowing request",
                        user_id
                    );
                    service.call(req).await
                }
                Ok(false) => {
                    // Determine which credit context to use
                    if let Some(org_id) = organization_id {
                        // Organization credit context - check organization credits
                        match Self::check_organization_credits(
                            &pool,
                            user_id,
                            org_id,
                            required_credits,
                            asset_count,
                        )
                        .await
                        {
                            Ok(()) => service.call(req).await,
                            Err(response) => Ok(req.into_response(response.map_into_boxed_body())),
                        }
                    } else {
                        // Not an old user, proceed with normal credit checks
                        match get_user_credit_allocation_by_user_id(pool.get_ref(), user_id).await {
                            Ok(Some(credit_allocation)) => {
                                // Check if user has sufficient credits
                                if credit_allocation.credits_remaining
                                    >= BigDecimal::from(required_credits)
                                {
                                    // User has sufficient credits, proceed with the request
                                    service.call(req).await
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
                                tracing::error!("Failed to get user credit allocation: {:?}", e);
                                let response =
                                    HttpResponse::InternalServerError().json(serde_json::json!({
                                        "error": "Internal Server Error",
                                        "message": "Failed to check credit allocation",
                                        "code": "INTERNAL_SERVER_ERROR",
                                    }));
                                Ok(req.into_response(response.map_into_boxed_body()))
                            }
                        }
                    }
                }
                Err(e) => {
                    // Database error while checking old user status
                    tracing::error!("Database error while checking old user status: {:?}", e);
                    let response = HttpResponse::InternalServerError().json(serde_json::json!({
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

impl<S> DynamicRetouchImagesGuardService<S> {
    /// Extract asset count and organization_id from request headers
    /// Returns (asset_count, organization_id)
    async fn extract_request_data(
        req: &ServiceRequest,
    ) -> Result<(i32, Option<uuid::Uuid>), HttpResponse<BoxBody>> {
        // Try to get asset count from custom header
        let asset_count = if let Some(header_value) = req.headers().get("x-asset-count") {
            if let Ok(count_str) = header_value.to_str() {
                if let Ok(count) = count_str.parse::<i32>() {
                    count
                } else {
                    1
                }
            } else {
                1
            }
        } else {
            // Default to 1 asset
            tracing::debug!(
                "No asset count header found, defaulting to 1 asset for credit calculation"
            );
            1
        };

        // Try to get organization_id from custom header
        let organization_id = crate::middleware::credits_guard_service::extract_organization_id_from_service_request(req);

        Ok((asset_count, organization_id))
    }

    /// Check if organization has sufficient credits and user has permission
    async fn check_organization_credits(
        pool: &web::Data<PgPool>,
        user_id: uuid::Uuid,
        org_id: uuid::Uuid,
        required_credits: i32,
        asset_count: i32,
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
                            "asset_count": asset_count,
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

/// Create a credits guard for generating creative from bundle (1 credit)
pub fn require_generate_creative_from_bundle() -> CreditsGuard {
    CreditsGuard::new(CreditsConsumption::GENERATE_CREATIVE_FROM_BUNDLE)
}

/// Create a credits guard for generating creative (1 credit)
pub fn require_generate_creative() -> CreditsGuard {
    CreditsGuard::new(CreditsConsumption::GENERATE_CREATIVE)
}

/// Create a credits guard for generating style (1 credit)
pub fn require_generate_style() -> CreditsGuard {
    CreditsGuard::new(CreditsConsumption::GENERATE_STYLE)
}

/// Create a credits guard for a specific operation type
pub fn require_credits_for_operation(operation: CreditOperation) -> CreditsGuard {
    CreditsGuard::new(operation.credits_changed())
}

/// Create a dynamic credits guard for retouching images based on asset count
pub fn require_dynamic_retouch_images() -> DynamicRetouchImagesGuard {
    DynamicRetouchImagesGuard::new()
}

/// Create a credits guard for vocal tour (1 credit)
pub fn require_create_vocal_tour() -> CreditsGuard {
    CreditsGuard::new(CreditsConsumption::VOCAL_TOUR)
}
