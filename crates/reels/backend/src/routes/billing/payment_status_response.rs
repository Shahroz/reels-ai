//! PaymentStatusResponse struct for API payment status endpoint responses.
//!
//! This structure represents recent payment completion information returned to clients,
//! including completion status, payment method details, completion timestamp, and session ID.
//! Used by the payment status API endpoint to check for recently completed payments
//! and enable frontend payment completion detection workflows.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaymentStatusResponse {
    pub is_completed: bool,
    pub payment_method: std::option::Option<std::string::String>,
    pub completed_at: std::option::Option<chrono::DateTime<chrono::Utc>>,
    pub session_id: std::option::Option<std::string::String>,
}
