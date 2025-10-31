//! Validates OAuth2 callback parameters for Google authentication.
//!
//! Handles validation of OAuth2 callback query parameters including error codes,
//! authorization codes, and state parameters with CSRF token and return URL.
//! Returns structured validation results or descriptive error messages.

/// Structure containing validated OAuth2 callback parameters
#[derive(Debug)]
pub struct ValidatedCallbackParams {
    pub auth_code: std::string::String,
    pub return_url: std::string::String,
    pub csrf_token: std::string::String,
}

/// OAuth2 callback validation errors with descriptive messages
#[derive(Debug)]
pub enum CallbackValidationError {
    OAuthError(std::string::String),
    MissingAuthCode,
    MissingState,
    InvalidStateFormat,
    InvalidStateEncoding,
    InvalidStateJson,
    MissingReturnUrl,
    EmptyCsrfToken,
}

impl CallbackValidationError {
    pub fn to_error_response(&self) -> actix_web::HttpResponse {
        match self {
            CallbackValidationError::OAuthError(error) => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": std::format!("OAuth2 authentication failed: {}", error)
                }))
            }
            CallbackValidationError::MissingAuthCode => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Missing authorization code"
                }))
            }
            CallbackValidationError::MissingState => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Missing OAuth state parameter"
                }))
            }
            CallbackValidationError::InvalidStateFormat => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid OAuth state parameter"
                }))
            }
            CallbackValidationError::InvalidStateEncoding => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid OAuth state encoding"
                }))
            }
            CallbackValidationError::InvalidStateJson => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid OAuth state format"
                }))
            }
            CallbackValidationError::MissingReturnUrl => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid OAuth state: missing return URL"
                }))
            }
            CallbackValidationError::EmptyCsrfToken => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid CSRF token"
                }))
            }
        }
    }
}

/// Validates OAuth2 callback parameters from Google authentication.
///
/// # Arguments
///
/// * `code` - Optional authorization code from Google
/// * `state` - Optional state parameter containing CSRF token and return URL
/// * `error` - Optional error message from Google OAuth2
///
/// # Returns
///
/// A `Result` containing validated parameters on success, or validation error on failure.
pub fn validate_callback_parameters(
    code: std::option::Option<&std::string::String>,
    state: std::option::Option<&std::string::String>,
    error: std::option::Option<&std::string::String>,
) -> std::result::Result<ValidatedCallbackParams, CallbackValidationError> {
    // Check for OAuth2 error first
    if let Some(error) = error {
        log::warn!("OAuth2 error received: {error}");
        return std::result::Result::Err(CallbackValidationError::OAuthError(error.clone()));
    }

    // Extract authorization code
    let auth_code = match code {
        Some(code) => code.clone(),
        None => {
            log::warn!("No authorization code received in callback");
            return std::result::Result::Err(CallbackValidationError::MissingAuthCode);
        }
    };

    // Validate CSRF token and extract return URL from state parameter
    let state_encoded = match state {
        Some(state) => state,
        None => {
            log::error!("No state parameter provided in OAuth callback");
            return std::result::Result::Err(CallbackValidationError::MissingState);
        }
    };

    // Decode the state parameter
    let state_bytes = match <base64::engine::GeneralPurpose as base64::Engine>::decode(&base64::engine::general_purpose::STANDARD, state_encoded) {
        std::result::Result::Ok(bytes) => bytes,
        std::result::Result::Err(e) => {
            log::error!("Failed to base64 decode state: {e}");
            return std::result::Result::Err(CallbackValidationError::InvalidStateFormat);
        }
    };

    let state_json = match std::string::String::from_utf8(state_bytes) {
        std::result::Result::Ok(json_str) => json_str,
        std::result::Result::Err(e) => {
            log::error!("Failed to decode state as UTF-8: {e}");
            return std::result::Result::Err(CallbackValidationError::InvalidStateEncoding);
        }
    };

    let state_data = match serde_json::from_str::<serde_json::Value>(&state_json) {
        std::result::Result::Ok(data) => data,
        std::result::Result::Err(e) => {
            log::error!("Failed to parse state JSON: {e}");
            return std::result::Result::Err(CallbackValidationError::InvalidStateJson);
        }
    };

    let csrf_token = state_data.get("csrf")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let return_url = state_data.get("return_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    if return_url.is_empty() {
        log::error!("No return_url found in state parameter");
        return std::result::Result::Err(CallbackValidationError::MissingReturnUrl);
    }
    
    if csrf_token.is_empty() {
        log::error!("Empty CSRF token in state parameter");
        return std::result::Result::Err(CallbackValidationError::EmptyCsrfToken);
    }

    std::result::Result::Ok(ValidatedCallbackParams {
        auth_code,
        return_url,
        csrf_token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_callback_parameters_oauth_error() {
        let result = validate_callback_parameters(
            std::option::Option::None,
            std::option::Option::None,
            std::option::Option::Some(&std::string::String::from("access_denied"))
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CallbackValidationError::OAuthError(err) => assert_eq!(err, "access_denied"),
            _ => panic!("Expected OAuthError"),
        }
    }

    #[test]
    fn test_validate_callback_parameters_missing_code() {
        let result = validate_callback_parameters(
            std::option::Option::None,
            std::option::Option::Some(&std::string::String::from("valid_state")),
            std::option::Option::None
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CallbackValidationError::MissingAuthCode => {},
            _ => panic!("Expected MissingAuthCode"),
        }
    }

    #[test]
    fn test_validate_callback_parameters_missing_state() {
        let result = validate_callback_parameters(
            std::option::Option::Some(&std::string::String::from("auth_code")),
            std::option::Option::None,
            std::option::Option::None
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CallbackValidationError::MissingState => {},
            _ => panic!("Expected MissingState"),
        }
    }

    #[test]
    fn test_validate_callback_parameters_success() {
        // Create a valid state parameter
        let state_data = serde_json::json!({
            "csrf": "test_csrf_token",
            "return_url": "https://app.narrativ.io/dashboard"
        });
        let state_encoded = <base64::engine::GeneralPurpose as base64::Engine>::encode(&base64::engine::general_purpose::STANDARD, state_data.to_string());
        
        let result = validate_callback_parameters(
            std::option::Option::Some(&std::string::String::from("auth_code123")),
            std::option::Option::Some(&state_encoded),
            std::option::Option::None
        );
        
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.auth_code, "auth_code123");
        assert_eq!(params.return_url, "https://app.narrativ.io/dashboard");
        assert_eq!(params.csrf_token, "test_csrf_token");
    }
} 