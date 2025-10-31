//! Handles JWT generation for short-lived, single-purpose task authentication.
//!
//! This is used for authenticating callbacks or workers for one-time research tasks,
//! ensuring they can only act on the specific resource they were authorized for.

use serde::{Deserialize, Serialize};
use std::env;
use std::env::VarError;
use uuid::Uuid;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};

/// Retrieves the task JWT secret from the environment.
///
/// This secret is specifically for signing short-lived tokens for background tasks.
pub fn get_task_jwt_secret() -> Result<String, VarError> {
    env::var("TASK_JWT_SECRET")
}

/// JWT claims for a one-time research task.
#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimeResearchClaims {
    /// The unique identifier for the one-time research task.
    pub one_time_research_id: Uuid,
    iat: i64,
    exp: i64,
}

/// Generates a JWT for a specific one-time research task.
///
/// The token is signed with a dedicated `TASK_JWT_SECRET` and has a 24-hour expiry.
///
/// # Arguments
///
/// * `research_id` - The UUID of the one-time research task to authorize.
///
/// # Returns
///
/// A `Result` containing the JWT string on success, or an error string on failure.
#[tracing::instrument(skip(research_id))]
pub fn generate_task_jwt(research_id: Uuid) -> Result<String, String> {
    let secret = match get_task_jwt_secret() {
        Ok(secret) => secret,
        Err(e) => {
            log::error!("TASK_JWT_SECRET not configured: {e}. Cannot create task JWT.");
            return Err(String::from("TASK_JWT_SECRET not configured."));
        }
    };

    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = OneTimeResearchClaims {
        one_time_research_id: research_id,
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };

    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|e| format!("Failed to generate task JWT: {e}"))
}

/// Verifies a task JWT and returns the claims.
///
/// # Arguments
///
/// * `token` - The JWT string to verify.
///
/// # Returns
///
/// A `Result` containing the `OneTimeResearchClaims` on success, or an error string on failure.
#[tracing::instrument(skip(token))]
pub fn verify_task_jwt(token: &str) -> Result<OneTimeResearchClaims, String> {
    let secret = match get_task_jwt_secret() {
        Ok(secret) => secret,
        Err(e) => {
            log::error!("TASK_JWT_SECRET not configured: {e}. Cannot verify task JWT.");
            return Err(String::from("TASK_JWT_SECRET not configured."));
        }
    };

    decode::<OneTimeResearchClaims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::new(Algorithm::HS256))
        .map(|data| data.claims)
        .map_err(|e| format!("Failed to verify task JWT: {e}"))
}
