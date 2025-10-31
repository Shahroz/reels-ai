//! Processes OAuth2 tokens and fetches user information from Google.
//!
//! Handles the OAuth2 token exchange flow including creating OAuth2 client,
//! exchanging authorization code for access token, and fetching user information
//! from Google's userinfo endpoint. Returns structured user data for account creation.

/// Structure containing processed OAuth2 token and user information
pub struct ProcessedOAuthToken {
    pub email: std::string::String,
    pub user_info: serde_json::Value,
}

/// OAuth2 token processing errors with descriptive messages
pub enum TokenProcessingError {
    ClientCreationError(std::string::String),
    TokenExchangeError(std::string::String),
    UserInfoFetchError(std::string::String),
    MissingEmail,
}

impl TokenProcessingError {
    pub fn to_error_response(&self) -> actix_web::HttpResponse {
        match self {
            TokenProcessingError::ClientCreationError(_) => {
                actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "OAuth2 configuration error"
                }))
            }
            TokenProcessingError::TokenExchangeError(_) => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to exchange authorization code"
                }))
            }
            TokenProcessingError::UserInfoFetchError(_) => {
                actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user information"
                }))
            }
            TokenProcessingError::MissingEmail => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "No email address provided by Google"
                }))
            }
        }
    }
}

/// Processes OAuth2 authorization code to retrieve user information.
///
/// # Arguments
///
/// * `auth_code` - The authorization code received from Google OAuth2 callback
///
/// # Returns
///
/// A `Result` containing processed token and user info on success, or processing error on failure.
pub async fn process_oauth_token(
    auth_code: &std::string::String,
) -> std::result::Result<ProcessedOAuthToken, TokenProcessingError> {
    let process_start = std::time::Instant::now();
    let code_length = auth_code.len();
    
    log::info!(
        "Processing OAuth token - Code length: {code_length}"
    );
    
    // Create OAuth2 client
    let client = match crate::auth::create_google_oauth_client::create_google_oauth_client() {
        std::result::Result::Ok(client) => {
            log::info!("OAuth2 client created successfully");
            client
        },
        std::result::Result::Err(e) => {
            log::error!("Failed to create OAuth2 client: {e}");
            return std::result::Result::Err(TokenProcessingError::ClientCreationError(e.to_string()));
        }
    };

    // Exchange authorization code for access token
    let token_response = match crate::auth::exchange_code::exchange_code(
        &client, 
        oauth2::AuthorizationCode::new(auth_code.clone())
    ).await {
        std::result::Result::Ok(token) => token,
        std::result::Result::Err(e) => {
            log::error!("Failed to exchange authorization code: {e}");
            return std::result::Result::Err(TokenProcessingError::TokenExchangeError(e.to_string()));
        }
    };

    // Fetch user information from Google
    let user_info = match crate::auth::fetch_user_info::fetch_user_info(
        oauth2::TokenResponse::access_token(&token_response).secret()
    ).await {
        std::result::Result::Ok(info) => info,
        std::result::Result::Err(e) => {
            log::error!("Failed to fetch user info from Google: {e}");
            return std::result::Result::Err(TokenProcessingError::UserInfoFetchError(e.to_string()));
        }
    };

    // Extract email from user info
    let email = match user_info.get("email").and_then(|e| e.as_str()) {
        Some(email) => email.to_lowercase(),
        None => {
            log::error!("No email found in Google user info: {user_info:?}");
            return std::result::Result::Err(TokenProcessingError::MissingEmail);
        }
    };

    let total_duration = process_start.elapsed().as_millis();
    log::info!(
        "OAuth token processing completed successfully - Total duration: {}ms, Email domain: {}", 
        total_duration,
        email.split('@').nth(1).unwrap_or("unknown")
    );
    
    std::result::Result::Ok(ProcessedOAuthToken {
        email,
        user_info,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_processing_error_responses() {
        let client_error = TokenProcessingError::ClientCreationError("config error".to_string());
        let response = client_error.to_error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        let exchange_error = TokenProcessingError::TokenExchangeError("exchange failed".to_string());
        let response = exchange_error.to_error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);

        let fetch_error = TokenProcessingError::UserInfoFetchError("fetch failed".to_string());
        let response = fetch_error.to_error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        let email_error = TokenProcessingError::MissingEmail;
        let response = email_error.to_error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_processed_oauth_token_structure() {
        // Test that the structure can be created
        let token = ProcessedOAuthToken {
            email: std::string::String::from("test@example.com"),
            user_info: serde_json::json!({"email": "test@example.com", "name": "Test User"}),
        };
        
        assert_eq!(token.email, "test@example.com");
        assert_eq!(token.user_info["email"], "test@example.com");
    }
} 