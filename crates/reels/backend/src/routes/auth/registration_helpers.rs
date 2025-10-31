//! Helper functions for user registration to improve code organization.
//!
//! Breaks down the registration process into smaller, focused functions
//! that handle specific aspects like password hashing, user creation,
//! and response generation. This improves readability and testability.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Added revision history, already follows guidelines (no use statements, has tests)

/// Result type for registration operations
type RegistrationResult<T> = std::result::Result<T, actix_web::HttpResponse>;

/// Hash a password using bcrypt with default cost
pub fn hash_password(password: &str) -> RegistrationResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
        log::error!("Password hashing failed: {e}");
        actix_web::HttpResponse::InternalServerError().json("Failed to process registration")
    })
}

/// Create a new user in the database
pub async fn create_user(
    pool: &sqlx::PgPool,
    postmark_client: &postmark::reqwest::PostmarkClient,
    email: &str,
    password_hash: &str,
) -> RegistrationResult<uuid::Uuid> {
    let user_id = crate::user_management::register_user(pool, postmark_client, email, password_hash, false)
        .await
        .map_err(|e| {
            log::error!("User registration failed: {e}");
            actix_web::HttpResponse::BadRequest().json("Registration failed")
        })?;
    
    // Create free subscription for new user
    if let Ok(billing_service) = crate::services::billing::billing_factory::get_billing_service() {
        if let Err(e) = billing_service.create_free_subscription(pool, user_id, email).await {
            log::warn!("Failed to create free subscription for new user {}: {}", user_id, e);
            // Don't fail registration if subscription creation fails
        } else {
            log::info!("Successfully created free subscription for new user: {}", user_id);
        }
    } else {
        log::warn!("Billing service not available, skipping free subscription creation for user: {}", user_id);
    }
    
    // Initialize credit reward tracking for new user
    if let Err(e) = crate::queries::credit_rewards::initialize_user_reward_tracking(pool, user_id).await {
        log::warn!("Failed to initialize credit reward tracking for new user {}: {}", user_id, e);
        // Don't fail registration if reward tracking initialization fails
    } else {
        log::info!("Successfully initialized credit reward tracking for new user: {}", user_id);
    }
    
    // Create personal organization for new user
    match crate::queries::organizations::create_personal_organization(
        pool,
        user_id,
        email,
        crate::app_constants::credits_constants::FREE_CREDITS,
    ).await {
        Ok(org) => {
            log::info!(
                "Successfully created personal organization {} for new user: {}",
                org.id,
                user_id
            );
        }
        Err(e) => {
            log::error!(
                "Failed to create personal organization for new user {}: {}",
                user_id,
                e
            );
            // Don't fail registration if personal org creation fails
            // The migration will catch any missing personal orgs
        }
    }
    
    Ok(user_id)
}

/// Fetch user details by ID
pub async fn fetch_user_details(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> RegistrationResult<crate::db::users::User> {
    match crate::user_management::find_user_by_id(pool, user_id).await {
        std::result::Result::Ok(std::option::Option::Some(user)) => std::result::Result::Ok(user),
        std::result::Result::Ok(std::option::Option::None) => {
            std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json("Failed to fetch user details"))
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to fetch user details: {e}");
            std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json("Failed to fetch user details"))
        }
    }
}

/// Extract User-Agent header from HTTP request
pub fn extract_user_agent(http_req: &actix_web::HttpRequest) -> std::option::Option<&str> {
    http_req
        .headers()
        .get(actix_web::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
}

/// Create successful registration response
pub fn create_success_response(
    token: String,
    user: crate::db::users::User,
    context: std::option::Option<crate::routes::auth::register_request::RegistrationContext>,
    user_agent: std::option::Option<&str>,
) -> actix_web::HttpResponse {
    let recommended_redirect = crate::routes::auth::determine_redirect_url::determine_redirect_url(context, user_agent);
    
    log::info!(
        "User registration successful (mobile: {}, context: {:?}), redirect: {}", 
        crate::routes::auth::is_mobile_device::is_mobile_device(user_agent),
        context,
        recommended_redirect
    );

    actix_web::HttpResponse::Created().json(serde_json::json!({
        "token": token,
        "user": crate::db::users::PublicUser::from(user),
        "recommended_redirect": recommended_redirect
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "test_password_123";
        let result = hash_password(password);
        assert!(result.is_ok(), "Password hashing should succeed");
        let hash = result.unwrap();
        assert!(!hash.is_empty(), "Hash should not be empty");
        assert_ne!(hash, password, "Hash should be different from password");
    }

    #[test]
    fn test_extract_user_agent_present() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("User-Agent", "Mozilla/5.0 Test"))
            .to_http_request();
        
        let user_agent = extract_user_agent(&req);
        assert_eq!(user_agent, std::option::Option::Some("Mozilla/5.0 Test"));
    }

    #[test]
    fn test_extract_user_agent_missing() {
        let req = actix_web::test::TestRequest::default().to_http_request();
        let user_agent = extract_user_agent(&req);
        assert_eq!(user_agent, std::option::Option::None);
    }
}