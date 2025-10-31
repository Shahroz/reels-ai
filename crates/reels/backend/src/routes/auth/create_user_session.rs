//! Creates user sessions and JWT tokens for OAuth2 authenticated users.
//!
//! Handles user account creation or lookup, validates user status, generates JWT tokens,
//! and constructs redirect URLs for successful OAuth2 authentication. Manages both
//! success and error redirect scenarios with appropriate URL construction.

/// Structure containing created user session information
pub struct CreatedUserSession {
    pub redirect_url: std::string::String,
    pub is_new_user: bool,
}

/// User session creation errors with descriptive messages
pub enum SessionCreationError {
    UserCreationError(std::string::String),
    UserDeactivated,
    JwtCreationError(std::string::String),
    InvalidReturnUrl(std::string::String),
}

impl SessionCreationError {
    pub fn to_error_response(&self, return_url: &str) -> actix_web::HttpResponse {
        match self {
            SessionCreationError::UserCreationError(_) => {
                actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to process user account"
                }))
            }
            SessionCreationError::UserDeactivated => {
                actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Account is deactivated. Please contact support."
                }))
            }
            SessionCreationError::JwtCreationError(e) => {
                log::error!("JWT creation failed: {e}");
                
                // Parse the return URL to extract the origin for error redirect
                let error_redirect = if let std::result::Result::Ok(mut error_url) = url::Url::parse(return_url) {
                    // Redirect to sign-in page with error, preserving the intended destination
                    error_url.set_path("/sign-in");
                    error_url.query_pairs_mut()
                        .clear()
                        .append_pair("error", "oauth_error")
                        .append_pair("redirect", return_url);
                    error_url.to_string()
                } else {
                    // Fallback if return URL is malformed
                    log::error!("Cannot parse return URL for error redirect: {return_url}");
                    std::string::String::from("http://localhost:5173/sign-in?error=oauth_error")
                };
                
                actix_web::HttpResponse::Found()
                    .append_header(("Location", error_redirect))
                    .finish()
            }
            SessionCreationError::InvalidReturnUrl(_) => {
                actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Invalid return URL format"
                }))
            }
        }
    }
}

/// Creates a user session with JWT token and redirect URL.
///
/// # Arguments
///
/// * `pool` - Database connection pool for user operations
/// * `email` - User's email address from OAuth2 provider
/// * `user_info` - Additional user information from OAuth2 provider
/// * `return_url` - URL where user should be redirected after authentication
/// * `user_agent` - Optional User-Agent header for device detection
///
/// # Returns
///
/// A `Result` containing session information on success, or session creation error on failure.
/// The session information includes whether this is a newly created user.
pub async fn create_user_session(
    pool: &sqlx::PgPool,
    email: &std::string::String,
    user_info: &serde_json::Value,
    return_url: &std::string::String,
    user_agent: Option<&str>,
    dub_service: Option<&dyn crate::services::dub::DubServiceTrait>,
) -> std::result::Result<CreatedUserSession, SessionCreationError> {
    // Find or create user
    let (user, is_new_user) = match crate::db::find_or_create_google_user::find_or_create_google_user(
        pool, email, user_info
    ).await {
        std::result::Result::Ok((user, is_new_user)) => (user, is_new_user),
        std::result::Result::Err(e) => {
            log::error!("Failed to find or create user for email {email}: {e}");
            return std::result::Result::Err(SessionCreationError::UserCreationError(e.to_string()));
        }
    };

    // Check user status
    if user.status == "deactivated" {
        return std::result::Result::Err(SessionCreationError::UserDeactivated);
    }

    // Ensure user has tracking for all active credit reward definitions
    if let Err(e) = crate::queries::credit_rewards::ensure_user_reward_tracking(
        pool,
        user.id,
    ).await {
        log::warn!("Failed to ensure credit reward tracking for OAuth user {}: {}", user.id, e);
    } else {
        log::info!("Successfully ensured credit reward tracking for OAuth user: {}", user.id);
    }

    // Generate JWT token
    let expiration = chrono::Utc::now() + chrono::Duration::days(30);
    let expiration_ts = expiration.timestamp() as u64;
    let claims = crate::auth::tokens::Claims {
        user_id: user.id,
        is_admin: user.is_admin,
        email: user.email.clone(),
        email_verified: user.email_verified,
        exp: expiration_ts,
        ..std::default::Default::default()
    };

    let token = match crate::auth::tokens::create_jwt(&claims) {
        std::result::Result::Ok(token) => {
            log::info!("Generated JWT token for OAuth user {}: {}...", user.id, &token[..token.len().min(50)]);
            token
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(SessionCreationError::JwtCreationError(e.to_string()));
        }
    };


    // Track lead event with Dub for new users (non-blocking)
    if is_new_user {
        if let Some(dub_service) = dub_service {
            // Extract dub_id from return_url for attribution
            let dub_id = if let Ok(url) = url::Url::parse(return_url) {
                url.query_pairs()
                    .find(|(key, _)| key == "dub_id")
                    .map(|(_, value)| value.to_string())
            } else {
                None
            };

            let mut lead_event = crate::services::dub::DubLeadEvent::new_signup(user.id, email.clone());
            lead_event.click_id = dub_id.clone();

            if let Err(e) = dub_service.track_lead_event(lead_event).await {
                // Log error but don't fail user session creation
                log::warn!("Failed to track lead event for OAuth user registration {}: {}", user.id, e);
            } else {
                log::info!("Successfully tracked lead event for OAuth user registration: {} (dub_id: {:?})", user.id, dub_id);
            }
        } else {
            log::debug!("No Dub service available for lead tracking");
        }
    }

    // For new users in real-estate context, modify redirect based on device type
    // BUT preserve invitation links - don't redirect to studio if user is accepting an invitation
    let final_return_url = if is_new_user && !return_url.contains("handle_invitation") {
        match url::Url::parse(return_url) {
            std::result::Result::Ok(mut url) => {
                // Extract the path context (e.g., "/real-estate" from "/real-estate/sign-up")
                // by removing the final path segment if it's a sign-up/sign-in route
                let original_path = url.path();
                let path_context = if original_path.ends_with("/sign-up") {
                    original_path.trim_end_matches("/sign-up")
                } else if original_path.ends_with("/sign-in") {
                    original_path.trim_end_matches("/sign-in")
                } else {
                    // If it's not a sign-up/sign-in route, use the full path
                    original_path
                };
                
                // Build target path by appending /studio to the context path
                let target_path = if path_context.is_empty() {
                    std::string::String::from("/studio")
                } else {
                    format!("{}/studio", path_context)
                };
                
                // Preserve important query parameters like dub_id for attribution
                let preserved_params: Vec<(String, String)> = url.query_pairs()
                    .filter(|(key, _)| key == "dub_id" || key == "utm_source" || key == "utm_medium" || key == "utm_campaign")
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect();
                
                url.set_path(&target_path);
                url.set_query(None); // Clear existing query params
                
                // Re-add preserved parameters
                if !preserved_params.is_empty() {
                    let mut query_pairs = url.query_pairs_mut();
                    for (key, value) in preserved_params {
                        query_pairs.append_pair(&key, &value);
                    }
                }
                
                log::info!("New user, redirecting to {target_path} with preserved attribution params");
                url.to_string()
            }
            std::result::Result::Err(_) => {
                // Fallback to the original return_url if parsing fails
                return_url.clone()
            }
        }
    } else {
        return_url.clone()
    };

    // Redirect directly to the original URL the user was trying to access
    // Add the token as a query parameter
    let redirect_url = match url::Url::parse(&final_return_url) {
        std::result::Result::Ok(mut url) => {
            url.query_pairs_mut().append_pair("token", &token);
            log::info!("OAuth redirect URL created: {url}");
            url.to_string()
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to parse return URL {final_return_url}: {e}");
            return std::result::Result::Err(SessionCreationError::InvalidReturnUrl(e.to_string()));
        }
    };
    
    log::info!("OAuth successful, redirecting to: {redirect_url} (new_user: {is_new_user})");
    
    std::result::Result::Ok(CreatedUserSession {
        redirect_url,
        is_new_user,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation_error_user_deactivated() {
        let error = SessionCreationError::UserDeactivated;
        let response = error.to_error_response("https://app.narrativ.io/dashboard");
        assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_session_creation_error_user_creation() {
        let error = SessionCreationError::UserCreationError(std::string::String::from("Database error"));
        let response = error.to_error_response("https://app.narrativ.io/dashboard");
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_session_creation_error_invalid_url() {
        let error = SessionCreationError::InvalidReturnUrl(std::string::String::from("Parse error"));
        let response = error.to_error_response("invalid-url");
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_created_user_session_structure() {
        // Test that the structure can be created
        let session = CreatedUserSession {
            redirect_url: std::string::String::from("https://app.narrativ.io/dashboard?token=jwt123"),
            is_new_user: true,
        };
        
        assert_eq!(session.redirect_url, "https://app.narrativ.io/dashboard?token=jwt123");
        assert!(session.is_new_user);
    }

    #[test]
    fn test_path_context_extraction() {
        // Test that path context is correctly extracted from return URLs
        
        // Test real-estate sign-up path extraction
        let real_estate_url = "http://localhost:5173/real-estate/sign-up";
        let parsed = url::Url::parse(real_estate_url).unwrap();
        let path = parsed.path();
        let context = if path.ends_with("/sign-up") {
            path.trim_end_matches("/sign-up")
        } else if path.ends_with("/sign-in") {
            path.trim_end_matches("/sign-in")
        } else {
            path
        };
        let target = if context.is_empty() {
            std::string::String::from("/studio")
        } else {
            format!("{}/studio", context)
        };
        assert_eq!(target, "/real-estate/studio");

        // Test root level sign-up path extraction
        let root_url = "http://localhost:5173/sign-up";
        let parsed = url::Url::parse(root_url).unwrap();
        let path = parsed.path();
        let context = if path.ends_with("/sign-up") {
            path.trim_end_matches("/sign-up")
        } else if path.ends_with("/sign-in") {
            path.trim_end_matches("/sign-in")
        } else {
            path
        };
        let target = if context.is_empty() {
            std::string::String::from("/studio")
        } else {
            format!("{}/studio", context)
        };
        assert_eq!(target, "/studio");

        // Test real-estate sign-in path extraction
        let real_estate_signin_url = "http://localhost:5173/real-estate/sign-in";
        let parsed = url::Url::parse(real_estate_signin_url).unwrap();
        let path = parsed.path();
        let context = if path.ends_with("/sign-up") {
            path.trim_end_matches("/sign-up")
        } else if path.ends_with("/sign-in") {
            path.trim_end_matches("/sign-in")
        } else {
            path
        };
        let target = if context.is_empty() {
            std::string::String::from("/studio")
        } else {
            format!("{}/studio", context)
        };
        assert_eq!(target, "/real-estate/studio");
    }
} 
