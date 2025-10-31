//! Magic link authentication request handler.
//!
//! Handles POST /auth/magic-link endpoint for requesting magic link emails.
//! Returns generic success message regardless of user existence to prevent
//! email enumeration attacks. Sends appropriate email based on user type.

/// Request body for magic link authentication.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MagicLinkRequest {
    /// User's email address
    pub email: std::string::String,
    /// Optional return URL to redirect after successful authentication
    #[serde(default)]
    pub return_url: std::option::Option<std::string::String>,
}

/// Response body for magic link request.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct MagicLinkResponse {
    /// Generic success message
    pub message: std::string::String,
}

/// Request a magic link for passwordless authentication.
///
/// # Security
///
/// - Always returns 200 OK with generic message to prevent email enumeration
/// - OAuth users receive guidance email instead of magic link
/// - Deactivated users receive no email but still return success
/// - Validates return_url to prevent open redirect attacks
#[utoipa::path(
    post,
    path = "/auth/magic-link",
    request_body = MagicLinkRequest,
    responses(
        (status = 200, description = "Magic link request processed", body = MagicLinkResponse),
        (status = 400, description = "Invalid request", body = serde_json::Value)
    ),
    tag = "Auth"
)]
#[actix_web::post("/magic-link")]
#[tracing::instrument(skip(pool, postmark_client))]
pub async fn request_magic_link(
    pool: actix_web::web::Data<sqlx::PgPool>,
    postmark_client: actix_web::web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
    req: actix_web::web::Json<MagicLinkRequest>,
) -> impl actix_web::Responder {
    // Validate email format
    if req.email.is_empty() {
        return actix_web::HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Email is required"}));
    }

    // Validate return_url if provided (prevent open redirects)
    if let std::option::Option::Some(ref url) = req.return_url {
        if let std::result::Result::Err(error_msg) = crate::auth::validate_return_url::validate_return_url(url) {
            log::warn!("Invalid return_url rejected in magic link request: {}", error_msg);
            return actix_web::HttpResponse::BadRequest()
                .json(serde_json::json!({"error": error_msg}));
        }
    }

    // Look up user by email (case-insensitive)
    let user_option = match crate::user_management::find_user_by_email(&pool, &req.email).await {
        std::result::Result::Ok(user) => user,
        std::result::Result::Err(e) => {
            // ERROR - Database failure (system issue)
            log::error!("Database error during magic link request: {}", e);
            // Return success to prevent enumeration
            return actix_web::HttpResponse::Ok().json(MagicLinkResponse {
                message: "If an account exists with this email, you will receive a login link shortly.".to_string(),
            });
        }
    };

    // Handle based on user existence and type
    match user_option {
        std::option::Option::None => {
            log::debug!("Magic link requested for non-existent email: {}", req.email);
        }
        std::option::Option::Some(user) => {
            if user.status == "deactivated" {
                log::warn!("Magic link requested for deactivated user: {}", user.id);
            } else if user.password_hash.is_none() {
                crate::routes::auth::handle_oauth_user_magic_link_request::handle_oauth_user_magic_link_request(&postmark_client, &user).await;
            } else {
                crate::routes::auth::handle_password_user_magic_link_request::handle_password_user_magic_link_request(
                    &postmark_client,
                    &user,
                    req.return_url.as_deref(),
                ).await;
            }
        }
    }

    // Always return success (prevent email enumeration)
    actix_web::HttpResponse::Ok().json(MagicLinkResponse {
        message: "If an account exists with this email, you will receive a login link shortly.".to_string(),
    })
}

