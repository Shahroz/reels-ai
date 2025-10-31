//! Validation limits for authentication input fields.
//!
//! Defines reasonable limits for user input to prevent abuse, ensure data integrity,
//! and provide consistent validation across the authentication system.

/// Maximum allowed length for email addresses (RFC 5321 specifies 320 characters)
pub const MAX_EMAIL_LENGTH: usize = 254;

/// Maximum allowed length for passwords (reasonable upper bound for security)
pub const MAX_PASSWORD_LENGTH: usize = 128;

/// Minimum allowed length for passwords (enforced by password validator)
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Maximum combined length for context strings
pub const MAX_CONTEXT_LENGTH: usize = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_limits_are_reasonable() {
        // Email limit should be within RFC standards
        assert!(MAX_EMAIL_LENGTH <= 320, "Email length should not exceed RFC 5321 limit");
        assert!(MAX_EMAIL_LENGTH >= 100, "Email length should allow reasonable addresses");

        // Password limits should be secure but usable
        assert!(MAX_PASSWORD_LENGTH >= 64, "Password length should allow strong passwords");
        assert!(MAX_PASSWORD_LENGTH <= 1024, "Password length should prevent abuse");
        assert!(MIN_PASSWORD_LENGTH >= 8, "Minimum password length should enforce security");

        // Context should be short and controlled
        assert!(MAX_CONTEXT_LENGTH <= 100, "Context should be constrained");
        assert!(MAX_CONTEXT_LENGTH >= 10, "Context should allow meaningful values");
    }
}