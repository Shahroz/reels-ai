use sqlx::{PgPool};
use tracing::instrument;
use serde::{Deserialize, Serialize};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder, HttpMessage};

use crate::middleware::auth::AuthenticatedUser;
use crate::services::billing::billing_config::BillingConfig;
use crate::services::billing::billing_factory::create_billing_service;
use crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCheckoutSessionRequest {
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
    /// Optional Dub attribution click ID for tracking conversions
    #[serde(default)]
    pub dub_id: Option<String>,
    /// Optional organization ID for purchasing credits for an organization
    /// If provided, the purchase will be for the organization instead of the user
    #[serde(default)]
    #[schema(format = "uuid", example = "550e8400-e29b-41d4-a716-446655440000")]
    pub organization_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateCheckoutSessionResponse {
    pub session_id: String,
    pub session_url: String,
}

#[utoipa::path(
    post,
    path = "/api/billing/checkout",
    tag = "Billing",
    request_body = CreateCheckoutSessionRequest,
    responses(
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
        (status = 200, description = "Checkout session created successfully", body = CreateCheckoutSessionResponse),
    ),
    security(
        ("jwt" = [])
    )
)]
#[post("/checkout")]
#[instrument(skip(pool,req))]
pub async fn create_checkout_session(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateCheckoutSessionRequest>,
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

    // Determine if this is an organization purchase
    let (customer_email, customer_type, org_id_for_metadata) = if let Some(org_id) = body.organization_id {
        // Verify user is a member of the organization
        match crate::queries::organizations::verify_organization_membership(pool.get_ref(), user_id, org_id).await {
            Ok(true) => {
                log::info!("User {} is authorized member of organization {}", user_id, org_id);
            }
            Ok(false) => {
                log::warn!("User {} attempted to purchase for organization {} without membership", user_id, org_id);
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You are not a member of this organization"
                }));
            }
            Err(e) => {
                log::error!("Failed to verify organization membership: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to verify organization membership"
                }));
            }
        }

        // Get organization details
        let organization = match crate::queries::organizations::find_organization_by_id(pool.get_ref(), org_id).await {
            Ok(Some(org)) => org,
            Ok(None) => {
                log::error!("Organization {} not found", org_id);
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Organization not found"
                }));
            }
            Err(e) => {
                log::error!("Failed to fetch organization: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch organization"
                }));
            }
        };

        // Get user email (we use the user's email for the Stripe customer for organizations too)
        let email = match crate::queries::billing::users::get_user_email(pool.get_ref(), user_id).await {
            Ok(email) => email,
            Err(e) => {
                log::error!("Failed to fetch user email: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user information"
                }));
            }
        };

        log::info!("Processing organization purchase for org {} ({})", organization.name, org_id);
        (email, "organization", Some(org_id))
    } else {
        // User purchase flow
        let email = match crate::queries::billing::users::get_user_email(pool.get_ref(), user_id).await {
            Ok(email) => email,
            Err(e) => {
                log::error!("Failed to fetch user email: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user information"
                }));
            }
        };

        // Get current credit allocation for validation (user only)
        let _current_credit_allocation = match get_user_credit_allocation_by_user_id(pool.get_ref(), user_id).await {
            Ok(allocation) => allocation,
            Err(e) => {
                log::error!("Failed to fetch credit allocation: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch credit allocation information"
                }));
            }
        };

        log::info!("Processing user purchase for user {}", user_id);
        (email, "user", None)
    };

    let config = BillingConfig::from_env();
    match create_billing_service(&config) {
        Ok(billing_service) => {
            // Fetch current pricing details from user subscription and credit allocation
            /*let _current_pricing_details = match &current_subscription {
                Some(subscription) => {
                    match billing_service.get_price(&subscription.stripe_price_id).await {
                        Ok(price) => Some(price),
                        Err(e) => {
                            log::error!("Failed to fetch current pricing details: {e}");
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to fetch current pricing details"
                            }));
                        }
                    }
                }
                None => None,
            };*/
            // Get the price details to determine the new plan type
            let price_details = match billing_service.get_price(&body.price_id).await {
                Ok(price) => price,
                Err(e) => {
                    log::error!("Failed to fetch price details: {e}");
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid price ID"
                    }));
                }
            };

            // Get the product details to determine the plan type
            /*let product_details = match billing_service.get_product(&price_details.product, false).await {
                Ok(product) => product,
                Err(e) => {
                    log::error!("Failed to fetch product details: {e}");
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid product"
                    }));
                }
            };*/

            // Extract the new plan type from product metadata
            /*let _new_plan_type = product_details.metadata.get("product_plan")
                .and_then(|v| v.as_str())
                .map(|s| StripePlanType::from_str(s))
                .unwrap_or(StripePlanType::Unknown);*/

            // Extract the new price type from price metadata
            let new_price_type = price_details.price_type
                .unwrap_or("unknown".to_string());

            // Extract credits from current pricing details
            /*let current_credits = current_pricing_details
                .as_ref()
                .and_then(|price| price.metadata.get("credits"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            // Extract credits from price metadata
            let purchase_credits = price_details.metadata.get("credits")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            // Perform validation checks based on current subscription
            if let Some(current_sub) = &current_subscription {
                let current_plan_type = current_sub.stripe_plan_type;
                
                // Check if user has an active subscription
                if current_sub.status == SubscriptionStatus::Active {
                    if let Some(current_allocation) = &current_credit_allocation {
                        log::info!("User current plan is {:?}, purchasing plan is {:?}, current credits is {:?}, purchase credits is {:?}", current_plan_type, new_plan_type, current_credits, purchase_credits);
                        // Case 1: Same plan purchase - check if user can purchase more credits
                        if is_same_credits(current_credits, purchase_credits) {
                            let available_space = current_allocation.credit_limit - current_allocation.credits_remaining;
                            if purchase_credits > available_space {
                                return HttpResponse::BadRequest().json(serde_json::json!({
                                    "error": "Please upgrade your plan to a higher tier to purchase more credits"
                                }));
                            }
                        }
                        // Case 2: Upgrade (lower plan to higher plan) - allowed, will update subscription and increase max limit
                        else if is_less_than_credits(current_credits, purchase_credits) {
                            // Upgrade is allowed - subscription will be updated, credits added, max limit increased
                            log::info!("User {} downgrading from {:?} to {:?}", user_id, current_plan_type, new_plan_type);
                        }
                        // Case 3: Downgrade (paid plan to paid plan) - allowed, will update subscription and reduce max limit
                        else if is_greater_than_credits(current_credits, purchase_credits) {
                            // Downgrade is allowed - subscription will be updated, credits kept, max limit reduced
                            log::info!("User {} upgrading from {:?} to {:?}", user_id, current_plan_type, new_plan_type);
                        }
                        // Case 4: Invalid transition (e.g., Free to Unknown, or other invalid combinations)
                        else {
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "error": "Invalid plan transition"
                            }));
                        }
                    }
                }
            }*/

            let mode = if new_price_type == "one_time" { "payment" } else { "subscription" };

            match billing_service.create_checkout_session_with_context(
                pool.get_ref(),
                user_id,
                &customer_email,
                &body.price_id,
                &body.success_url,
                &body.cancel_url,
                mode,
                body.dub_id.as_deref(),
                customer_type,
                org_id_for_metadata,
            ).await {
                Ok(session) => {
                    log::info!("Successfully created checkout session {} for user {}", session.session_id, user_id);
                    HttpResponse::Ok().json(CreateCheckoutSessionResponse {
                        session_id: session.session_id,
                        session_url: session.session_url,
                    })
                }
                Err(e) => {
                    log::error!("Failed to create checkout session: {e}");
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create checkout session",
                        "message": e.to_string()
                    }))
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCheckoutSessionRequest {
    pub stripe_checkout_id: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/billing/checkout/update",
    tag = "Billing",
    request_body = UpdateCheckoutSessionRequest,
    responses(
        (status = 200, description = "Checkout session updated successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
#[post("/update")]
#[instrument(skip(req))]
pub async fn update_checkout_session(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<UpdateCheckoutSessionRequest>,
) -> impl Responder {
    // Extract user ID from authenticated user
    let _user_id = if let Some(auth_user) = req.extensions().get::<AuthenticatedUser>() {
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
            match billing_service.update_checkout_session_status(
                pool.get_ref(),
                &body.stripe_checkout_id,
                &body.status,
                body.metadata.clone(),
            ).await {
                Ok(_) => {
                    log::info!("Successfully updated checkout session {} status to {}", 
                               body.stripe_checkout_id, body.status);
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": "Checkout session updated successfully"
                    }))
                }
                Err(e) => {
                    log::error!("Failed to update checkout session: {e}");
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to update checkout session",
                        "message": e.to_string()
                    }))
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