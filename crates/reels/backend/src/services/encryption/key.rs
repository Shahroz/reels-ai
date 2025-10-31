//! Provides a function to load the application's encryption key from the environment.
//!
//! This module centralizes the logic for retrieving and parsing the `ENCRYPTION_KEY`
//! environment variable. The key must be a 64-character hex-encoded string,
//! which corresponds to 32 bytes of entropy for AES-256.

/// Loads the 32-byte encryption key from the `ENCRYPTION_KEY` environment variable.
///
/// The key is expected to be a 64-character hex-encoded string.
///
/// # Returns
///
/// A `Result` containing a 32-byte array (`[u8; 32]`) on success, or a `String`
/// error message if the variable is not set or parsing fails.
pub fn load_encryption_key() -> std::result::Result<[u8; 32], std::string::String> {
    let key_hex = std::env::var("ENCRYPTION_KEY")
        .map_err(|_| "ENCRYPTION_KEY environment variable not set".to_string())?;

    let key_bytes = hex::decode(key_hex)
        .map_err(|e| format!("Failed to decode hex for ENCRYPTION_KEY: {e}"))?;

    if key_bytes.len() != 32 {
        return std::result::Result::Err("ENCRYPTION_KEY must be 32 bytes (64 hex characters) long".to_string());
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    std::result::Result::Ok(key)
} 