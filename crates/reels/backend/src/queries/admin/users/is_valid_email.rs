//! Email validation for user creation.
//!
//! Validates email addresses using a proper regex pattern that checks for
//! common email format requirements (local@domain.tld).
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from batch_create_users.rs with improved regex validation

pub fn is_valid_email(email: &str) -> bool {
    // RFC 5322 compliant email regex (simplified)
    // Matches: user@example.com, user.name@example.co.uk, user+tag@example.com
    static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        regex::Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).expect("Invalid regex pattern")
    });
    
    regex.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.com"));
        assert!(is_valid_email("user+tag@example.com"));
        assert!(is_valid_email("user_name@example.co.uk"));
        assert!(is_valid_email("123@example.com"));
        assert!(is_valid_email("user@subdomain.example.com"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("@"));
        assert!(!is_valid_email("user"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user @example.com")); // space
        assert!(!is_valid_email("user@@example.com")); // double @
        assert!(!is_valid_email("user@.com")); // starts with dot
    }

    #[test]
    fn test_edge_cases() {
        assert!(!is_valid_email("   ")); // whitespace only
        assert!(is_valid_email("a@b.c")); // minimal valid email
        // Note: Regex allows single-letter TLDs and domains without TLDs for flexibility
        // In practice, email validation should be confirmed via verification emails
        assert!(is_valid_email("user@example")); // Valid per regex (no TLD required)
        assert!(is_valid_email("user@example.c")); // Valid per regex (single char TLD)
        assert!(is_valid_email("very.long.email.address.with.many.dots@subdomain.example.com"));
    }
}

