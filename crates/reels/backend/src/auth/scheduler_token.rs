//! Generates and defines structures for internal scheduler service JWTs.
//!
//! This module centralizes the creation of JWTs used by Google Cloud Scheduler
//! to authorize calls to internal API endpoints, such as `run_infinite_research`.
//! It ensures that tokens are created consistently and correctly, including all
//! required claims like `iat` and `exp`.

use crate::utils;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The payload for the scheduler JWT, identifying the task to be run.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InfiniteResearchClaims {
    /// The ID of the user who owns the research task.
    pub user_id: Uuid,
    /// The ID of the infinite research task to execute.
    pub infinite_research_id: Uuid,
}

/// Generates a long-lived JWT for a scheduler job to authorize itself.
///
/// This token uses the generic JWT generation utility, ensuring it has the
/// required `iat` and `exp` claims. The expiry is set to 10 years to be
/// effectively permanent for the job's lifetime.
pub fn generate_scheduler_jwt(user_id: Uuid, research_id: Uuid) -> String {
    // In a real production scenario, the secret should be handled more robustly.
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        log::warn!("JWT_SECRET not set, using default secret for scheduler token. This is insecure.");
        "secret".to_string()
    });

    let claims = InfiniteResearchClaims {
        user_id,
        infinite_research_id: research_id,
    };

    // 10 years in days.
    const TEN_YEARS_IN_DAYS: &str = "3650d";

    match utils::jwt::generate_jwt_token(&secret, TEN_YEARS_IN_DAYS, &claims) {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to generate scheduler JWT: {e}");
            // Fallback to an empty string, which will fail later but prevents a panic.
            String::new()
        }
    }
}