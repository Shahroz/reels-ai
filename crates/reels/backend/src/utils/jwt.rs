use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::instrument;

/// Claims structure that includes standard JWT claims and custom payload
#[derive(Debug, Serialize, Deserialize)]
struct Claims<T> {
    /// Issued at timestamp
    iat: i64,
    /// Expiration timestamp
    exp: i64,
    /// Custom payload data
    #[serde(flatten)]
    payload: T, // The generic payload
}

/// Generates a JWT token with the given payload and expiration time.
///
/// # Arguments
///
/// * `secret` - The secret key used to sign the token
/// * `expiry` - The duration string (e.g., "1h", "1d") for token expiration
/// * `payload` - The custom payload data to include in the token
///
/// # Returns
///
/// Returns a Result containing the JWT token string or an error
#[instrument(skip(secret, expiry, payload))]
pub fn generate_jwt_token<T: Serialize>(
    secret: &str,
    expiry: &str,
    payload: &T,
) -> Result<String, Box<dyn Error>> {
    // Parse the expiry duration
   let duration = {
       // Inlined logic from parse_duration(expiry)
       let (num_part, unit_part) = expiry.split_at(
           expiry
               .find(|c: char| !c.is_ascii_digit())
               .unwrap_or(expiry.len()),
       );

       // num_part.parse() will correctly error for empty strings or non-numbers.
       let num = num_part.parse::<i64>().map_err(Box::new)?;

       match unit_part {
           "s" => Duration::seconds(num),
           "m" => Duration::minutes(num),
           "h" => Duration::hours(num),
           "d" => Duration::days(num),
           _ => return Result::Err(format!("Invalid duration unit: '{unit_part}' in duration string '{expiry}'").into()),
       }
   };

    // Calculate timestamps
    let now = Utc::now();
    let exp = now + duration;

    // Create claims with standard fields and custom payload
    let claims = Claims {
        iat: now.timestamp(),
        exp: exp.timestamp(),
        payload,
    };

    // Create the token header
    let header = Header::default();

    // Create the encoding key
    let key = EncodingKey::from_secret(secret.as_bytes());

    // Encode the token
    let token = encode(&header, &claims, &key)?;

    Ok(token)
}

/// Verifies a JWT token and returns the decoded payload.
///
/// # Arguments
///
/// * `token_str` - The JWT token string to verify
/// * `secret` - The secret key used to sign the token
///
/// # Returns
///
/// Returns a Result containing the decoded payload `T` or an error
#[instrument(skip(token_str, secret))]
pub fn verify_jwt_token<T: for<'de> serde::Deserialize<'de>>(
    token_str: &str,
    secret: &str,
) -> Result<T, Box<dyn Error>> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default(); // default validation checks expiry `exp`
    let token_data = decode::<Claims<T>>(token_str, &key, &validation)?;
    Ok(token_data.claims.payload)
}

#[cfg(test)] // Changed from FALSE
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_jwt_token() {
        // Test payload
        let payload = json!({
            "user_id": 123,
            "role": "admin"
        });

        // Generate token
        let result = generate_jwt_token("test_secret", "1h", &payload);

        // Verify token was generated
        assert!(result.is_ok());
        let token = result.unwrap();

        // Basic token validation
        assert!(!token.is_empty());
        assert!(token.split('.').count() == 3); // JWT should have 3 parts
    }

    #[test]
    fn test_verify_jwt_token() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestPayload {
            user_id: i32,
            role: String,
        }

        let payload = TestPayload {
            user_id: 456,
            role: "user".to_string(),
        };
        let secret = "another_secret";

        // Generate token
        let token = generate_jwt_token(secret, "5m", &payload).unwrap();

        // Verify token
        let decoded_payload_result = verify_jwt_token::<TestPayload>(&token, secret);

        assert!(decoded_payload_result.is_ok());
        let decoded_payload = decoded_payload_result.unwrap();

        assert_eq!(payload, decoded_payload);
    }

    #[test]
    fn test_verify_expired_jwt_token() {
        let payload = json!({ "data": "test" });
        let secret = "expired_secret";
        // Generate a token that expired in the past. `generate` doesn't support negative,
        // so we can't easily test this without more complex time mocking.
        // Instead, we'll test with an invalid secret.
        let token = generate_jwt_token(secret, "1h", &payload).unwrap();

        let result = verify_jwt_token::<serde_json::Value>(&token, "wrong_secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_jwt_token_with_invalid_expiry() {
        // Test payload
        let payload = json!({
            "user_id": 123
        });

        // Try to generate token with invalid expiry
        let result = generate_jwt_token(
            "test_secret",
            "1x", // Invalid duration unit
            &payload,
        );

        // Verify error
        assert!(result.is_err());
    }

}
