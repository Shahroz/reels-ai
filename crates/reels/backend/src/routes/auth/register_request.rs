//! Request struct for user registration.
//!
//! Defines the expected JSON payload for registering a new user.

/// Valid registration contexts that determine vocal tour flow
#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RegistrationContext {
    RealEstate,
}

impl RegistrationContext {
    /// Returns the string representation for comparison and logging
    pub fn as_str(&self) -> &'static str {
        match self {
            RegistrationContext::RealEstate => "real-estate",
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    /// Optional context indicating the source of the registration
    pub context: Option<RegistrationContext>,
    /// Optional Dub attribution click ID for tracking conversions
    #[serde(default)]
    pub dub_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_context_serialization() {
        // Test that serde correctly deserializes kebab-case
        let json = r#"{"context": "real-estate"}"#;
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        let context: RegistrationContext = serde_json::from_value(parsed["context"].clone()).unwrap();
        assert_eq!(context, RegistrationContext::RealEstate);
        assert_eq!(context.as_str(), "real-estate");
    }

    #[test]
    fn test_registration_context_invalid() {
        // Test that invalid context strings are rejected
        let json = r#"{"context": "invalid-context"}"#;
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        let result: Result<RegistrationContext, _> = serde_json::from_value(parsed["context"].clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_register_request_with_context() {
        let json = r#"{"email": "test@example.com", "password": "password123", "context": "real-estate"}"#;
        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.context, Some(RegistrationContext::RealEstate));
    }

    #[test]
    fn test_register_request_without_context() {
        let json = r#"{"email": "test@example.com", "password": "password123"}"#;
        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.context, None);
    }
}
