//! Validates style name input according to business rules.
//!
//! This function checks style names for emptiness, whitespace-only content,
//! and length constraints. Returns validation errors as HTTP responses
//! for immediate use in API handlers. Ensures consistent validation
//! across all style operations.

/// Validates style name input according to business rules
/// 
/// Checks for empty names, whitespace-only names, and excessive length.
/// Returns Ok(()) if valid, Err(HttpResponse) if invalid for immediate API response.
pub fn validate_style_name(name: &str) -> std::result::Result<(), actix_web::HttpResponse> {
    // Check for empty name
    if name.is_empty() {
        return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Style name cannot be empty."),
            }
        ));
    } else if name.trim().is_empty() { // Check for whitespace-only name
        return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Style name cannot be only whitespace."),
            }
        ));
    } else if name.len() > 255 { // Check for overly long name (255 character limit)
        return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Style name cannot exceed 255 characters."),
            }
        ));
    } else {
        std::result::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_valid_style_names() {
        // Test normal valid names
        let result = super::validate_style_name("My Style");
        assert!(result.is_ok());
        
        let result = super::validate_style_name("a");
        assert!(result.is_ok());
        
        let result = super::validate_style_name("Style123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_style_name() {
        // Test empty name rejection
        let result = super::validate_style_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_style_name() {
        // Test whitespace-only name rejection
        let result = super::validate_style_name("   ");
        assert!(result.is_err());
        
        let result = super::validate_style_name("\t\n  \r");
        assert!(result.is_err());
    }

    #[test]
    fn test_overly_long_style_name() {
        // Test name length limit (255 characters)
        let long_name = "a".repeat(256);
        let result = super::validate_style_name(&long_name);
        assert!(result.is_err());
        
        // Test exactly 255 characters (should be valid)
        let max_length_name = "a".repeat(255);
        let result = super::validate_style_name(&max_length_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        // Test names with leading/trailing whitespace but valid content
        let result = super::validate_style_name("  valid name  ");
        assert!(result.is_ok());
        
        // Test special characters
        let result = super::validate_style_name("Style-Name_123!");
        assert!(result.is_ok());
        
        // Test unicode characters
        let result = super::validate_style_name("Стиль名前スタイル");
        assert!(result.is_ok());
    }
} 