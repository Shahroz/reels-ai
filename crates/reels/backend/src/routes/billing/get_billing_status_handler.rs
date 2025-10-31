//! HTTP handler for retrieving user billing status.
//!
//! This handler processes GET requests to /api/billing/status and returns comprehensive
//! billing information including individual trial/subscription status. As of 2025-10-17,
//! organization-based access sharing has been removed. Access now requires individual
//! credits, trial status, or active subscription.
//! 
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Removed organization access check (organization hack removed)
//! - 2025-09-17T20:45:00Z @AI: Created during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[utoipa::path(
    get,
    path = "/api/billing/status",
    tag = "Billing",
    responses(
        (status = 200, description = "Billing status retrieved successfully", body = crate::routes::billing::billing_status_response::BillingStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::get("/status")]
#[tracing::instrument(skip(pool, req))]
pub async fn get_billing_status_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    req: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let user_id = extract_user_id_from_request(&req);
    let user_id = match user_id {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(response) => return response,
    };

    match crate::services::trial_service::get_billing_status::get_billing_status(&pool, user_id).await {
        std::result::Result::Ok(billing_status) => {
            let individual_access = calculate_individual_access(&billing_status);
            let (trial_status, days_remaining) = determine_trial_status_response(billing_status.trial_status);
            // Organization access hack removed - always false
            let organization_access = false;
            let can_access_app = individual_access;
            let access_source = determine_access_source(individual_access, organization_access);

            #[allow(deprecated)]
            let response = crate::routes::billing::billing_status_response::BillingStatusResponse {
                trial_status,
                days_remaining,
                subscription_status: billing_status.subscription_status,
                has_active_subscription: billing_status.has_active_subscription,
                can_access_app,
                stripe_customer_id: billing_status.stripe_customer_id,
                has_organization_access: organization_access,
                access_source,
            };

            actix_web::HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to get billing status for user {user_id}: {e}");
            actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": "Failed to retrieve billing status"
            }))
        }
    }
}

fn extract_user_id_from_request(req: &actix_web::HttpRequest) -> std::result::Result<uuid::Uuid, actix_web::HttpResponse> {
    if let std::option::Option::Some(auth_user) = actix_web::HttpMessage::extensions(req).get::<crate::middleware::auth::AuthenticatedUser>() {
        let user_id = match auth_user {
            crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
            crate::middleware::auth::AuthenticatedUser::ApiKey(user_id) => *user_id,
        };
        std::result::Result::Ok(user_id)
    } else {
        std::result::Result::Err(actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        })))
    }
}

fn determine_trial_status_response(trial_status: crate::services::trial_service::trial_status::TrialStatus) -> (std::string::String, std::option::Option<i64>) {
    match trial_status {
        crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining } => {
            ("active".to_string(), std::option::Option::Some(days_remaining))
        }
        crate::services::trial_service::trial_status::TrialStatus::Expired => {
            ("expired".to_string(), std::option::Option::None)
        }
        crate::services::trial_service::trial_status::TrialStatus::NotStarted => {
            ("not_started".to_string(), std::option::Option::None)
        }
    }
}

fn calculate_individual_access(billing_status: &crate::services::trial_service::billing_status::BillingStatus) -> bool {
    matches!(billing_status.trial_status, 
        crate::services::trial_service::trial_status::TrialStatus::Active { .. }) || 
        billing_status.has_active_subscription
}

/// **DEPRECATED:** Organization access check removed as of 2025-10-17.
/// 
/// This function is kept for reference but is no longer called.
/// Organization membership hack has been removed.
#[deprecated(
    since = "1.0.0",
    note = "Organization membership hack removed as of 2025-10-17. Function no longer used."
)]
#[allow(dead_code)]
async fn check_organization_access(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> bool {
    #[allow(deprecated)]
    match crate::services::trial_service::has_organization_access::has_organization_access(pool, user_id).await {
        std::result::Result::Ok(access) => access,
        std::result::Result::Err(e) => {
            log::error!("Failed to check organization access for user {user_id}: {e}");
            false
        }
    }
}

fn determine_access_source(individual_access: bool, organization_access: bool) -> crate::routes::billing::access_source::AccessSource {
    match (individual_access, organization_access) {
        (true, true) => crate::routes::billing::access_source::AccessSource::Both,
        (true, false) => crate::routes::billing::access_source::AccessSource::Individual,
        (false, true) => crate::routes::billing::access_source::AccessSource::Organization,
        (false, false) => crate::routes::billing::access_source::AccessSource::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_trial_status_response_active() {
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 5 };
        let (status, days) = determine_trial_status_response(trial_status);
        assert_eq!(status, "active");
        assert_eq!(days, std::option::Option::Some(5));
    }

    #[test]
    fn test_determine_trial_status_response_expired() {
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::Expired;
        let (status, days) = determine_trial_status_response(trial_status);
        assert_eq!(status, "expired");
        assert_eq!(days, std::option::Option::None);
    }

    #[test]
    fn test_determine_trial_status_response_not_started() {
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::NotStarted;
        let (status, days) = determine_trial_status_response(trial_status);
        assert_eq!(status, "not_started");
        assert_eq!(days, std::option::Option::None);
    }

    #[test]
    fn test_determine_access_source_both() {
        let access_source = determine_access_source(true, true);
        assert!(matches!(access_source, crate::routes::billing::access_source::AccessSource::Both));
    }

    #[test]
    fn test_determine_access_source_individual_only() {
        let access_source = determine_access_source(true, false);
        assert!(matches!(access_source, crate::routes::billing::access_source::AccessSource::Individual));
    }

    #[test]
    fn test_determine_access_source_organization_only() {
        let access_source = determine_access_source(false, true);
        assert!(matches!(access_source, crate::routes::billing::access_source::AccessSource::Organization));
    }

    #[test]
    fn test_determine_access_source_none() {
        let access_source = determine_access_source(false, false);
        assert!(matches!(access_source, crate::routes::billing::access_source::AccessSource::None));
    }

    #[test]
    fn test_calculate_individual_access_with_active_trial() {
        let billing_status = crate::services::trial_service::billing_status::BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 3 },
            subscription_status: "trial".to_string(),
            has_active_subscription: false,
            stripe_customer_id: std::option::Option::None,
        };
        assert!(calculate_individual_access(&billing_status));
    }

    #[test]
    fn test_calculate_individual_access_with_subscription() {
        let billing_status = crate::services::trial_service::billing_status::BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Expired,
            subscription_status: "active".to_string(),
            has_active_subscription: true,
            stripe_customer_id: std::option::Option::Some("cus_test".to_string()),
        };
        assert!(calculate_individual_access(&billing_status));
    }

    #[test]
    fn test_calculate_individual_access_no_access() {
        let billing_status = crate::services::trial_service::billing_status::BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Expired,
            subscription_status: "expired".to_string(),
            has_active_subscription: false,
            stripe_customer_id: std::option::Option::None,
        };
        assert!(!calculate_individual_access(&billing_status));
    }
}
