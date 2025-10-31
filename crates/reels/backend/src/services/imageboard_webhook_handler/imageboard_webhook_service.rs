use uuid::Uuid;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageboardSigClaims {
    pub sub: String,
    pub webhook_id: String, // Unique identifier for this token
    pub board_id: String,
}


pub fn generate_client_signature(
    board_id: &Uuid,
) -> Result<String, String> {
    let salt = std::env::var("IMAGEBOARD_WEBHOOK_JWT_SECRET").map_err(|_| "IMAGEBOARD_WEBHOOK_JWT_SECRET not set".to_string())?;
    // Generate a unique UUID for this token to ensure uniqueness
    let webhook_id = Uuid::new_v4().to_string();
    let claims = ImageboardSigClaims {
        sub: "imageboard_sig".to_string(),
        webhook_id,
        board_id: board_id.to_string(),
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(salt.as_bytes()))
        .map_err(|e| format!("Failed to encode JWT: {}", e))
}

pub fn verify_and_extract_from_signature(token: &str) -> Result<(Uuid, Uuid), String> {
    let salt = std::env::var("IMAGEBOARD_WEBHOOK_JWT_SECRET").map_err(|_| "IMAGEBOARD_WEBHOOK_JWT_SECRET not set".to_string())?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // No expiry validation
    validation.validate_nbf = false; // No not-before validation
    // Remove exp and nbf from required claims since we don't use them
    validation.required_spec_claims.remove("exp");
    validation.required_spec_claims.remove("nbf");
    let data = decode::<ImageboardSigClaims>(token, &DecodingKey::from_secret(salt.as_bytes()), &validation)
        .map_err(|e| format!("Invalid signature: {}", e))?;
    let claims = data.claims;
    let webhook_id = Uuid::parse_str(&claims.webhook_id).map_err(|_| "Invalid webhook_id".to_string())?;
    let board_id = Uuid::parse_str(&claims.board_id).map_err(|_| "Invalid board_id".to_string())?;
    Ok((webhook_id, board_id))
}


