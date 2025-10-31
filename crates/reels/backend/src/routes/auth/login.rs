//! Handler for user login.
//!
//! Verifies credentials and returns a JWT token and user info.
use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::routes::auth::login_request::LoginRequest;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    params(
        ("utm_source" = Option<String>, Query, description = "UTM source parameter"),
        ("utm_medium" = Option<String>, Query, description = "UTM medium parameter"),
        ("utm_campaign" = Option<String>, Query, description = "UTM campaign parameter"),
        ("utm_term" = Option<String>, Query, description = "UTM term parameter"),
        ("utm_content" = Option<String>, Query, description = "UTM content parameter"),
        ("dub_id" = Option<String>, Query, description = "Dub.co tracking ID"),
        ("ref" = Option<String>, Query, description = "Referral parameter"),
        ("referrer" = Option<String>, Query, description = "HTTP referrer parameter"),
        ("gclid" = Option<String>, Query, description = "Google Ads click ID"),
        ("fbclid" = Option<String>, Query, description = "Facebook click ID"),
        ("msclkid" = Option<String>, Query, description = "Microsoft/Bing click ID"),
        ("ttclid" = Option<String>, Query, description = "TikTok click ID"),
        ("twclid" = Option<String>, Query, description = "Twitter click ID"),
        ("li_fat_id" = Option<String>, Query, description = "LinkedIn click ID"),
    ),
    responses(
        (status = 200, description = "Login successful", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::post("/login")]
#[instrument(skip(pool, req, http_req, session_manager))]
pub async fn login(
    pool: actix_web::web::Data<sqlx::PgPool>,
    req: actix_web::web::Json<crate::routes::auth::login_request::LoginRequest>,
    http_req: actix_web::HttpRequest,
    session_manager: actix_web::web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> impl actix_web::Responder {
    let processing_start = std::time::Instant::now();
    
    // Extract request context for event tracking
    #[cfg(feature = "events")]
    let mut request_context = extract_request_context_for_login(&http_req, &session_manager).await;
    
    let user_option = match crate::user_management::find_user_by_email(&pool.get_ref(), &req.email).await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Database error during login for {}: {}", req.email, e);
            return actix_web::HttpResponse::InternalServerError()
                .json("Login failed due to server error");
        }
    };

    let user = match user_option {
        Some(u) => u,
        None => {
            log::warn!("Login attempt failed for non-existent email: {}", req.email);
            return actix_web::HttpResponse::Unauthorized().json("Invalid email or password");
        }
    };

    if user.status == "deactivated" {
        return actix_web::HttpResponse::Unauthorized().json("The account is not active please contact support");
    }

     // Ensure user has tracking for all active credit reward definitions
     if let Err(e) = crate::queries::credit_rewards::ensure_user_reward_tracking(
        &pool,
        user.id,
    ).await {
        log::warn!("Failed to ensure credit reward tracking for user {}: {}", user.id, e);
    } else {
        log::info!("Successfully ensured credit reward tracking for user: {}", user.id);
    }

    let is_old_user = user.created_at < crate::app_constants::credits_constants::OLD_USER_CUTOFF_DATE;
    let user_subscription_status = user.subscription_status.clone().unwrap_or(SubscriptionStatus::Trial.as_str().to_string());

    // If user is old, then check for subscription status, if active, then return true, otherwise return false
    if is_old_user {
        if user_subscription_status == SubscriptionStatus::Trial.as_str() || user_subscription_status == SubscriptionStatus::Trialing.as_str() || user_subscription_status == SubscriptionStatus::Expired.as_str() || user_subscription_status == SubscriptionStatus::Canceled.as_str() {
            // Creating a free user credit allocation (no Stripe subscription needed)
            if let Ok(billing_service) = crate::services::billing::billing_factory::get_billing_service() {
                if let Err(e) = billing_service.create_free_subscription(pool.get_ref(), user.id, &user.email).await {
                    log::error!("Failed to create free credit allocation for user {}: {}", user.id, e);
                } else {
                    log::info!("Successfully created free credit allocation for user: {}", user.id);
                }
            }
        }
    }

    // Check if user has a password (not OAuth user)
    let password_hash = match &user.password_hash {
        Some(hash) => hash,
        None => {
            log::warn!("Password login attempted for OAuth user: {}", user.id);
            return actix_web::HttpResponse::Unauthorized()
                .json("This account uses OAuth authentication. Please use 'Sign in with Google'.");
        }
    };

    match bcrypt::verify(&req.password, password_hash) {
        Ok(true) => {
           let expiration = chrono::Utc::now() + chrono::Duration::hours(24*30);
           let expiration_ts = expiration.timestamp() as u64;
           let claims = crate::auth::tokens::Claims {
               user_id: user.id,
               is_admin: user.is_admin,
               email: user.email.clone(),
               email_verified: user.email_verified,
               exp: expiration_ts,
               ..Default::default()
           };
           match crate::auth::tokens::create_jwt(&claims) {
                Ok(token) => {
                    // Log successful login event
                    #[cfg(feature = "events")]
                    {
                        // Update request context with user_id and session_id after successful login
                        request_context.user_id = Some(user.id);
                        
                        // Get session ID using session manager - SAME PATTERN AS VOCAL TOUR
                        request_context.session_id = match session_manager.get_or_create_session(user.id).await {
                            Ok(session) => Some(session),
                            Err(e) => {
                                log::warn!("Failed to get session for user {}: {}", user.id, e);
                                None
                            }
                        };
                        
                        let _ = crate::services::events_service::auth_events::log_user_login_successful(
                            &pool,
                            user.id,
                            &request_context,
                            processing_start,
                        ).await;
                    }
                    
                    actix_web::HttpResponse::Ok().json(serde_json::json!({
                    "token": token,
                    "user": crate::db::users::PublicUser::from(user)
                    }))
                },
                Err(e) => {
                    log::error!("JWT creation failed for user {}: {}", user.id, e);
                    actix_web::HttpResponse::InternalServerError()
                        .json("Login failed due to token generation error")
                }
            }
        }
        Ok(false) => {
            log::warn!(
                "Login attempt failed for user {} due to invalid credentials.",
                user.id
            );
            actix_web::HttpResponse::Unauthorized().json("Invalid email or password")
        }
        Err(e) => {
            log::error!("Password verification error for user {}: {}", user.id, e);
            actix_web::HttpResponse::InternalServerError().json("Login failed due to server error")
        }
    }
}

/// Extract request context for login event tracking
#[cfg(feature = "events")]
async fn extract_request_context_for_login(
    http_req: &actix_web::HttpRequest,
    session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> crate::services::events_service::request_context::RequestData {
    // Extract basic request info
    let method = http_req.method().to_string();
    let path = http_req.path().to_string();
    let query_string = http_req.query_string().to_string();
    let scheme = if http_req.connection_info().scheme() == "https" { "https" } else { "http" };
    let host = http_req.connection_info().host().to_string();
    let full_url = format!("{}://{}{}", scheme, host, path);
    
    // Extract headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in http_req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    // Extract IP address
    let connection_info = http_req.connection_info();
    let ip_address = connection_info.realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .map(|addr| addr.split(':').next().unwrap_or(addr).to_string());
    
    // Extract user agent
    let user_agent = headers.get("user-agent").cloned();
    
    // For login, we don't have user_id yet to get session, but we'll populate it in the event
    let session_id = None;
    
    crate::services::events_service::request_context::RequestData {
        method,
        path,
        full_url,
        query_string,
        headers,
        query_params: serde_json::json!({}),
        user_agent,
        ip_address,
        real_ip: None,
        forwarded_for: None,
        scheme: scheme.to_string(),
        host,
        port: None,
        http_version: format!("{:?}", http_req.version()),
        content_type: None,
        content_length: None,
        content_encoding: None,
        accept_language: None,
        accept_encoding: None,
        request_body: None,
        request_body_size: None,
        request_body_truncated: false,
        user_registration_date: None,
        cookies: std::collections::HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        user_id: None, // Will be populated after successful login
        session_id,
    }
}
