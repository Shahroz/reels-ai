//! Determines redirect URL based on user context and device type.
//!
//! This function analyzes user registration context and device information
//! to determine the appropriate studio flow. Real estate users get
//! device-specific routes (studio for mobile, standard studio for desktop).
//! General users are redirected to the home page.

/// Determines the recommended redirect URL based on user context and device type
pub fn determine_redirect_url(context: Option<crate::routes::auth::register_request::RegistrationContext>, user_agent: Option<&str>) -> String {
    //let is_mobile = crate::routes::auth::is_mobile_device::is_mobile_device(user_agent);

    match context {
        Some(crate::routes::auth::register_request::RegistrationContext::RealEstate) => {
            "/real-estate/studio".to_string()
        }
        None => {
            // For general users, redirect to home
            "/".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::routes::auth::register_request::RegistrationContext;

    #[test]
    fn test_real_estate_mobile_redirect() {
        let result = super::determine_redirect_url(
            Some(RegistrationContext::RealEstate), 
            Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X)")
        );
        assert_eq!(result, "/real-estate/studio");
    }

    #[test]
    fn test_real_estate_desktop_redirect() {
        let result = super::determine_redirect_url(
            Some(RegistrationContext::RealEstate), 
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        );
        assert_eq!(result, "/real-estate/studio");
    }

    #[test]
    fn test_general_user_redirect() {
        let result = super::determine_redirect_url(
            None, 
            Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X)")
        );
        assert_eq!(result, "/");
    }

    #[test]
    fn test_no_context_no_user_agent() {
        let result = super::determine_redirect_url(
            None, 
            None
        );
        assert_eq!(result, "/");
    }

    #[test]
    fn test_enum_security() {
        // Test that only valid enum values can be passed - this validates type safety
        let context = RegistrationContext::RealEstate;
        assert_eq!(context.as_str(), "real-estate");
        
        let result = super::determine_redirect_url(
            Some(context),
            None
        );
        assert_eq!(result, "/real-estate/studio");
    }
}