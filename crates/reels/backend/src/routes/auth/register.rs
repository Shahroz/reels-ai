//! Handler for user registration.
//!
//! Processes registration requests and returns a JWT token on success.

/// Validates email format using a simple regex pattern
fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email) && !email.starts_with('@') && !email.ends_with('@')
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    request_body = crate::routes::auth::register_request::RegisterRequest,
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
        (status = 201, description = "Registration successful", body = crate::routes::auth::login_response::LoginResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::post("/register")]
#[tracing::instrument(skip(pool, http_req, req, dub_service, session_manager))]
pub async fn register(
    pool: actix_web::web::Data<sqlx::PgPool>,
    postmark_client: actix_web::web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
    http_req: actix_web::HttpRequest,
    req: actix_web::web::Json<crate::routes::auth::register_request::RegisterRequest>,
    dub_service: actix_web::web::Data<dyn crate::services::dub::DubServiceTrait>,
    session_manager: actix_web::web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> impl actix_web::Responder {
    let processing_start = std::time::Instant::now();
    
    // Extract request context for event tracking
    #[cfg(feature = "events")]
    let mut request_context = extract_request_context(&http_req, &session_manager).await;
    
    // Log registration started event
    #[cfg(feature = "events")]
    {
        let _ = crate::services::events_service::auth_events::log_user_registration_started(
            &pool,
            &req.email,
            &request_context,
        ).await;
    }
    
    // Basic empty field validation
    if req.email.is_empty() || req.password.is_empty() {
        #[cfg(feature = "events")]
        {
            let _ = crate::services::events_service::auth_events::log_user_registration_failed(
                &pool,
                &req.email,
                "empty_fields",
                &["Email and password cannot be empty".to_string()],
                &request_context,
                processing_start,
            ).await;
        }
        return actix_web::HttpResponse::BadRequest().json("Email and password cannot be empty");
    }

    // Email format validation
    if !is_valid_email(&req.email) {
        #[cfg(feature = "events")]
        {
            let _ = crate::services::events_service::auth_events::log_user_registration_failed(
                &pool,
                &req.email,
                "invalid_email_format",
                &["Invalid email format".to_string()],
                &request_context,
                processing_start,
            ).await;
        }
        return actix_web::HttpResponse::BadRequest().json("Invalid email format");
    }

    // Input length validation to prevent abuse
    if req.email.len() > crate::routes::auth::validation_limits::MAX_EMAIL_LENGTH {
        return actix_web::HttpResponse::BadRequest().json("Email address is too long");
    }
    
    if req.password.len() > crate::routes::auth::validation_limits::MAX_PASSWORD_LENGTH {
        return actix_web::HttpResponse::BadRequest().json("Password is too long");
    }

    // Password strength validation
    if let std::result::Result::Err(msg) = crate::utils::password_validator::validate_password(&req.password) {
        #[cfg(feature = "events")]
        {
            let _ = crate::services::events_service::auth_events::log_user_registration_failed(
                &pool,
                &req.email,
                "weak_password",
                &[msg.clone()],
                &request_context,
                processing_start,
            ).await;
        }
        return actix_web::HttpResponse::BadRequest().json(msg);
    }

    // Hash password using helper function
    let password_hash = match crate::routes::auth::registration_helpers::hash_password(&req.password) {
        std::result::Result::Ok(hash) => hash,
        std::result::Result::Err(response) => return response,
    };

    // Create user using helper function
    let user_id = match crate::routes::auth::registration_helpers::create_user(&pool, &postmark_client, &req.email, &password_hash).await {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(response) => {
            #[cfg(feature = "events")]
            {
                let _ = crate::services::events_service::auth_events::log_user_registration_failed(
                    &pool,
                    &req.email,
                    "database_error",
                    &["Failed to create user account".to_string()],
                    &request_context,
                    processing_start,
                ).await;
            }
            return response;
        },
    };

    // Create JWT token directly
    let expiration = chrono::Utc::now() + chrono::Duration::hours(24);
    let expiration_ts = expiration.timestamp() as u64;
    let claims = crate::auth::tokens::Claims {
        user_id,
        is_admin: false,
        email: req.email.clone(),
        email_verified: false, // New users haven't verified their email yet
        exp: expiration_ts,
        ..std::default::Default::default()
    };
    
    let token = match crate::auth::tokens::create_jwt(&claims) {
        std::result::Result::Ok(t) => t,
        std::result::Result::Err(e) => {
            log::error!("JWT creation failed: {e}");
            return actix_web::HttpResponse::InternalServerError().json("Failed to generate token");
        }
    };

    // Fetch user details using helper function
    let user = match crate::routes::auth::registration_helpers::fetch_user_details(&pool, user_id).await {
        std::result::Result::Ok(u) => u,
        std::result::Result::Err(response) => return response,
    };


    // Debug: Log the registration request dub_id
    log::info!("Registration request received - dub_id: {:?}", req.dub_id);
    
    // Track lead event with Dub (non-blocking) - only if attribution is available
    let mut lead_event = crate::services::dub::DubLeadEvent::new_signup(user_id, req.email.clone());
    if let Some(click_id) = req.dub_id.as_ref() {
        lead_event.click_id = Some(click_id.clone());
        log::info!("Tracking registration lead event for user {} with attribution: {}", user_id, click_id);
        
        if let Err(e) = dub_service.track_lead_event(lead_event).await {
            // Log error but don't fail user registration
            log::warn!("Failed to track attributed lead event for user registration {}: {}", user_id, e);
        } else {
            log::info!("Successfully tracked attributed lead event for user registration: {}", user_id);
        }
    } else {
        log::info!("Skipping Dub lead tracking for user {} - no attribution data available", user_id);
    }

    // Log registration completed event
    #[cfg(feature = "events")]
    {
        // Update request context with user_id and session_id after successful registration
        request_context.user_id = Some(user_id);
        
        // Get session ID using session manager - SAME PATTERN AS VOCAL TOUR
        request_context.session_id = match session_manager.get_or_create_session(user_id).await {
            Ok(session) => Some(session),
            Err(e) => {
                log::warn!("Failed to get session for user {}: {}", user_id, e);
                None
            }
        };
        
        let _ = crate::services::events_service::auth_events::log_user_registration_completed(
            &pool,
            user_id,
            &req.email,
            &request_context,
            processing_start,
        ).await;
    }

    // Extract user agent and create success response
    let user_agent = crate::routes::auth::registration_helpers::extract_user_agent(&http_req);
    crate::routes::auth::registration_helpers::create_success_response(token, user, req.context, user_agent)
}

/// Extract request context for event tracking
#[cfg(feature = "events")]
async fn extract_request_context(
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
    
    // For registration, we don't have a user_id yet, so we can't get a session
    // The session will be created after successful registration
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
        user_id: None, // No user_id during registration
        session_id,
    }
}

#[cfg(test)]
mod tests {
    // Note: Device detection tests are available in the create_user_session module
    // The functionality is tested there to avoid compilation issues
}
