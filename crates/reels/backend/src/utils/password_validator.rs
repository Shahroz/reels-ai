//! Validates passwords against the application's security policy.

/// Validates that a password meets the required security criteria:
/// - Minimum length of 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one numeric digit
/// - At least one special character
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long.".to_string());
    }
    if !password.chars().any(char::is_uppercase) {
        return Err("Password must contain at least one uppercase letter.".to_string());
    }
    if !password.chars().any(char::is_lowercase) {
        return Err("Password must contain at least one lowercase letter.".to_string());
    }
    if !password.chars().any(char::is_numeric) {
        return Err("Password must contain at least one number.".to_string());
    }
    // A set of standard special characters
    let special_chars = r#"!@#$%^&*()_+-=[]{};':"|,.<>/?~"#;
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err("Password must contain at least one special character.".to_string());
    }
    Ok(())
}
