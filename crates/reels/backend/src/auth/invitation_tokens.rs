//! Module for generating and validating JWT-based invitation tokens.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationClaims {
    pub exp: usize,         // Expiration timestamp (seconds since Unix epoch)
    pub iat: usize,         // Issued at timestamp (seconds since Unix epoch)
    pub iss: String,        // Issuer (e.g., "narrativ.com")
    pub aud: String,        // Audience (e.g., "narrativ_invitation")
    pub org_id: Uuid,       // Organization ID for the invitation
    pub email: String,      // Recipient's email address
    pub role: String,       // Role to be assigned (e.g., "member")
}

/// Generates a JWT for an organization invitation.
pub fn generate_invitation_token(
    org_id: Uuid,
    recipient_email: &str,
    role_to_assign: &str,
    issuer: &str,
    audience: &str,
    secret: &str,
    duration_hours: i64, // Added duration_hours parameter
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    // Use duration_hours for expiration
    let expiration_time = now + Duration::hours(duration_hours);

    let claims = InvitationClaims {
        exp: expiration_time.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: issuer.to_string(),
        aud: audience.to_string(),
        org_id,
        email: recipient_email.to_string(),
        role: role_to_assign.to_string(),
    };

    // Use passed-in secret
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Validates an invitation JWT and returns the claims if valid.
pub fn validate_invitation_token(
    token: &str,
    issuer: &str,
    audience: &str,
    secret: &str, // Added secret parameter
) -> Result<InvitationClaims, jsonwebtoken::errors::Error> {
    // Use passed-in secret
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);

    decode::<InvitationClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
   const INVITATION_TOKEN_DURATION_DAYS: i64 = 7;

    #[test]
    fn test_generate_and_validate_invitation_token() {
        let test_secret = "test_secret_for_invitations_valid_run_local"; // Local test secret

        let org_id = Uuid::new_v4();
        let email = "test@example.com";
        let role = "member";
        let issuer = "test_issuer";
        let audience = "test_audience";

        let token_result = generate_invitation_token(org_id, email, role, issuer, audience, test_secret, INVITATION_TOKEN_DURATION_DAYS * 24);
        assert!(token_result.is_ok(), "Token generation failed: {:?}", token_result.err());
        let token = token_result.unwrap();
        assert!(!token.is_empty());

        let claims_result = validate_invitation_token(&token, issuer, audience, test_secret);
        assert!(claims_result.is_ok(), "Token validation failed: {:?}. Token: {}. Used Secret: {}", claims_result.err(), token, test_secret);

        let claims = claims_result.unwrap();
        assert_eq!(claims.org_id, org_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role);
        assert_eq!(claims.iss, issuer);
        assert_eq!(claims.aud, audience);

        let now_plus_6_days = (Utc::now() + Duration::days(6)).timestamp() as usize;
        let now_plus_8_days = (Utc::now() + Duration::days(8)).timestamp() as usize;
        assert!(claims.exp > now_plus_6_days);
        assert!(claims.exp < now_plus_8_days);

        assert!(validate_invitation_token(&token, "wrong_issuer", audience, test_secret).is_err());
        assert!(validate_invitation_token(&token, issuer, "wrong_audience", test_secret).is_err());
        assert!(validate_invitation_token(&token, issuer, audience, "wrong_secret").is_err());
    }

    #[test]
    fn test_expired_token() {
        let test_secret = "test_secret_for_invitations_expired_run_local"; // Local test secret

        let org_id = Uuid::new_v4();
        let email = "expired@example.com";
        let role = "member";
        let issuer = "test_issuer";
        let audience = "test_audience";

        let now = Utc::now();
        let expired_claims = InvitationClaims {
            exp: (now - Duration::days(1)).timestamp() as usize,
            iat: (now - Duration::days(INVITATION_TOKEN_DURATION_DAYS + 1)).timestamp() as usize,
            iss: issuer.to_string(),
            aud: audience.to_string(),
            org_id,
            email: email.to_string(),
            role: role.to_string(),
        };

        let expired_token_result = encode(
            &Header::default(),
            &expired_claims,
            &EncodingKey::from_secret(test_secret.as_ref()),
        );
        assert!(expired_token_result.is_ok(), "Expired token generation failed: {:?}", expired_token_result.err());
        let expired_token = expired_token_result.unwrap();

        let validation_result = validate_invitation_token(&expired_token, issuer, audience, test_secret);
        assert!(validation_result.is_err());
        if let Err(e) = validation_result {
            assert_eq!(e.kind(), &jsonwebtoken::errors::ErrorKind::ExpiredSignature, "Expected ExpiredSignature, got {:?}. Token: {}", e.kind(), expired_token);
        } else {
            panic!("Token should be expired");
        }

        // Test that validating with the wrong secret still gives InvalidSignature, not ExpiredSignature
        let wrong_secret_validation_result = validate_invitation_token(&expired_token, issuer, audience, "a_completely_different_secret");
        assert!(wrong_secret_validation_result.is_err());
        if let Err(e) = wrong_secret_validation_result {
            assert_eq!(e.kind(), &jsonwebtoken::errors::ErrorKind::InvalidSignature, "Expected InvalidSignature with wrong secret, got {:?}. Token: {}", e.kind(), expired_token);
        } else {
            panic!("Token validation with wrong secret should fail with InvalidSignature");
        }
    }
}