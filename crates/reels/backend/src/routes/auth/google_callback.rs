//! Handler for processing Google OAuth2 callback.
//!
//! Exchanges the authorization code for an access token, fetches user information
//! from Google, and creates or logs in the user. Returns a JWT token on success.
//! Refactored to use modular helper functions for better maintainability.

#[derive(serde::Deserialize)]
pub struct GoogleCallbackQuery {
    code: std::option::Option<std::string::String>,
    state: std::option::Option<std::string::String>,
    error: std::option::Option<std::string::String>,
}

#[utoipa::path(
    get,
    path = "/auth/google/callback",
    tag = "Auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code from Google"),
        ("state" = Option<String>, Query, description = "CSRF state parameter"),
        ("error" = Option<String>, Query, description = "Error from Google OAuth2")
    ),
    responses(
        (status = 200, description = "Login successful", body = crate::routes::auth::login_response::LoginResponse),
        (status = 400, description = "Bad request - missing or invalid parameters"),
        (status = 401, description = "Authentication failed"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::get("/google/callback")]
#[tracing::instrument(skip(pool, _req, query, dub_service))]
pub async fn google_callback(
    pool: actix_web::web::Data<sqlx::PgPool>,
    _req: actix_web::HttpRequest,
    query: actix_web::web::Query<GoogleCallbackQuery>,
    dub_service: actix_web::web::Data<dyn crate::services::dub::DubServiceTrait>,
) -> impl actix_web::Responder {
    log::info!(
        "OAuth callback received - Has code: {}, Has state: {}, Has error: {}", 
        query.code.is_some(),
        query.state.is_some(), 
        query.error.is_some()
    );
    
    if let Some(error) = &query.error {
        log::error!("OAuth callback received error from Google: {error}");
    }
    
    // Validate callback parameters
    let validated_params = match crate::routes::auth::validate_callback_parameters::validate_callback_parameters(
        query.code.as_ref(),
        query.state.as_ref(),
        query.error.as_ref(),
    ) {
        std::result::Result::Ok(params) => {
            // Log only the domain from return URL for security
            let return_url_domain = url::Url::parse(&params.return_url)
                .map(|u| u.host_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|_| "invalid_url".to_string());
            
            log::info!(
                "OAuth callback parameters validated - Return URL domain: {return_url_domain}"
            );
            params
        },
        std::result::Result::Err(e) => {
            log::error!("OAuth callback parameter validation failed: {e:?}");
            return e.to_error_response();
        }
    };

    // Process OAuth token and fetch user info
    let processed_token = match crate::routes::auth::process_oauth_token::process_oauth_token(
        &validated_params.auth_code
    ).await {
        std::result::Result::Ok(token) => token,
        std::result::Result::Err(e) => {
            return e.to_error_response();
        }
    };

    // Extract User-Agent header for device detection
    let user_agent = _req.headers()
        .get(actix_web::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok());

    // Create user session and generate redirect
    match crate::routes::auth::create_user_session::create_user_session(
        &pool,
        &processed_token.email,
        &processed_token.user_info,
        &validated_params.return_url,
        user_agent,
        Some(dub_service.as_ref()),
    ).await {
        std::result::Result::Ok(session) => {
            actix_web::HttpResponse::Found()
                .append_header(("Location", session.redirect_url))
                .finish()
        }
        std::result::Result::Err(e) => {
            e.to_error_response(&validated_params.return_url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_callback_query_deserialize() {
        // Test that the query structure can be created
        let query = GoogleCallbackQuery {
            code: std::option::Option::Some(std::string::String::from("auth_code123")),
            state: std::option::Option::Some(std::string::String::from("state_data")),
            error: std::option::Option::None,
        };
        
        assert_eq!(query.code.unwrap(), "auth_code123");
        assert_eq!(query.state.unwrap(), "state_data");
        assert!(query.error.is_none());
    }

    #[test]
    fn test_google_callback_query_with_error() {
        // Test error case
        let query = GoogleCallbackQuery {
            code: std::option::Option::None,
            state: std::option::Option::None,
            error: std::option::Option::Some(std::string::String::from("access_denied")),
        };
        
        assert!(query.code.is_none());
        assert!(query.state.is_none());
        assert_eq!(query.error.unwrap(), "access_denied");
    }
}

 