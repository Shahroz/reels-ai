//! Magic link JWT claims structure.
//!
//! Defines the claims payload for magic link authentication tokens.
//! These tokens are short-lived (15 minutes) and single-use via token_version.
//! Unlike regular session tokens, magic link tokens include the token_version
//! for single-use enforcement and have a much shorter expiration time.

/// JWT claims structure for magic link authentication tokens.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone, utoipa::ToSchema)]
pub struct MagicLinkClaims {
    /// The unique identifier of the user requesting authentication
    pub user_id: uuid::Uuid,
    /// The user's email address for verification
    pub email: std::string::String,
    /// Token version for single-use enforcement
    pub token_version: i32,
    /// Token type identifier (always "magic-link")
    pub token_type: std::string::String,
    /// Token expiration time as Unix timestamp (seconds since epoch)
    pub exp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_link_claims_serialization() {
        let user_id = uuid::Uuid::new_v4();
        let claims = MagicLinkClaims {
            user_id,
            email: "user@example.com".to_string(),
            token_version: 0,
            token_type: "magic-link".to_string(),
            exp: 1640995200, // 2022-01-01 00:00:00 UTC
        };

        let serialized = serde_json::to_string(&claims).expect("Serialization should succeed");
        let deserialized: MagicLinkClaims = serde_json::from_str(&serialized).expect("Deserialization should succeed");
        
        assert_eq!(claims, deserialized);
    }

    #[test]
    fn test_magic_link_claims_fields() {
        let user_id = uuid::Uuid::new_v4();
        let claims = MagicLinkClaims {
            user_id,
            email: "test@example.com".to_string(),
            token_version: 5,
            token_type: "magic-link".to_string(),
            exp: 1640995200,
        };

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.token_version, 5);
        assert_eq!(claims.token_type, "magic-link");
        assert_eq!(claims.exp, 1640995200);
    }
}

