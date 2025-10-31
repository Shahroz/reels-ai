//! JWT claims structure for authentication and authorization.
//!
//! Defines the standard claims used in JWT tokens including user identification,
//! admin privileges, feature flags, expiration time, and impersonation support.
//! The structure supports both regular user sessions and admin impersonation scenarios.
//! All timestamps are stored as Unix epoch seconds for compatibility with JWT standards.

/// JWT claims structure containing user authentication and authorization data.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct Claims {
    /// The unique identifier of the authenticated user
    pub user_id: uuid::Uuid,
    /// Whether the user has administrative privileges
    pub is_admin: bool,
    /// The email address of the authenticated user
    pub email: std::string::String,
    /// Whether the user's email has been verified
    pub email_verified: bool,
    /// Optional feature flags enabled for this user session
    pub feature_flags: std::option::Option<std::vec::Vec<std::string::String>>,
    /// Token expiration time as Unix timestamp (seconds since epoch)
    pub exp: u64,
    /// ID of the admin performing impersonation, if any
    pub admin_id: std::option::Option<uuid::Uuid>,
    /// Whether this token represents an impersonation session
    pub is_impersonating: std::option::Option<bool>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_claims_default() {
        let claims = super::Claims::default();
        assert_eq!(claims.user_id, uuid::Uuid::nil());
        assert_eq!(claims.is_admin, false);
        assert_eq!(claims.email, "");
        assert_eq!(claims.email_verified, false);
        assert_eq!(claims.feature_flags, std::option::Option::None);
        assert_eq!(claims.exp, 0);
        assert_eq!(claims.admin_id, std::option::Option::None);
        assert_eq!(claims.is_impersonating, std::option::Option::None);
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = uuid::Uuid::new_v4();
        let admin_id = uuid::Uuid::new_v4();
        let claims = super::Claims {
            user_id,
            is_admin: true,
            email: "test@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::Some(std::vec!["feature1".to_string(), "feature2".to_string()]),
            exp: 1640995200, // 2022-01-01 00:00:00 UTC
            admin_id: std::option::Option::Some(admin_id),
            is_impersonating: std::option::Option::Some(true),
        };

        let serialized = serde_json::to_string(&claims).expect("Serialization should succeed");
        let deserialized: super::Claims = serde_json::from_str(&serialized).expect("Deserialization should succeed");
        
        assert_eq!(claims, deserialized);
    }
}
