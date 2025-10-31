//! Handler for initiating Google OAuth2 login.
//!
//! Redirects users to Google's authorization server to begin the OAuth2 flow.
//! Uses the OAuth state parameter to preserve the complete return URL.



#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct GoogleLoginQuery {
    /// Complete URL where user should be redirected after successful OAuth
    /// e.g., "https://app.narrativ.io/real-estate" or "http://localhost:5173/research/new" or "http://re.bounti.ai/real-estate"
    return_url: std::option::Option<std::string::String>,
}

#[utoipa::path(
    get,
    path = "/auth/google",
    tag = "Auth",
    params(
        ("return_url" = Option<String>, Query, description = "Complete URL to redirect after successful OAuth")
    ),
    responses(
        (status = 302, description = "Redirect to Google OAuth2 authorization"),
        (status = 400, description = "Bad request - invalid return URL"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::get("/google")]
#[tracing::instrument]
pub async fn google_login(req: actix_web::HttpRequest, query: actix_web::web::Query<GoogleLoginQuery>) -> impl actix_web::Responder {
    // Validate return URL
    let return_url = match &query.return_url {
        std::option::Option::Some(url) => match crate::auth::validate_return_url::validate_return_url(url) {
            std::result::Result::Ok(validated_url) => validated_url,
            std::result::Result::Err(error_msg) => {
                return actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_msg
                }));
            }
        },
        std::option::Option::None => {
            log::warn!("No return_url provided in OAuth request");
            return actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                "error": "return_url parameter is required"
            }));
        }
    };

    // Create OAuth2 client and generate authorization URL
    match crate::auth::create_google_oauth_client::create_google_oauth_client() {
        std::result::Result::Ok(client) => {
            let (auth_url, csrf_token) = crate::auth::generate_auth_url::generate_auth_url(&client);
            let state_encoded = crate::auth::encode_oauth_state::encode_oauth_state(&csrf_token, &return_url);
            let final_auth_url = crate::auth::build_google_auth_url::build_google_auth_url(auth_url, &state_encoded, &req);
            
            log::info!("Redirecting to Google OAuth2 with return URL: {return_url}");
            
            actix_web::HttpResponse::Found()
                .append_header(("Location", final_auth_url.to_string()))
                .finish()
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to create OAuth2 client: {e}");
            actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth2 configuration error"
            }))
        }
    }
} 