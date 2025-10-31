//! BillingStatusResponse struct for API billing status endpoint responses.
//!
//! This structure represents the complete billing status information returned to clients,
//! including trial status, subscription details, and access source.
//! Used by the billing status API endpoint to provide comprehensive user access information.
//! 
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Deprecated has_organization_access field (organization hack removed)
//! - 2025-09-17T20:45:00Z @AI: Created during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BillingStatusResponse {
    pub trial_status: std::string::String,
    pub days_remaining: std::option::Option<i64>,
    pub subscription_status: std::string::String,
    pub has_active_subscription: bool,
    pub can_access_app: bool,
    pub stripe_customer_id: std::option::Option<std::string::String>,
    /// **DEPRECATED:** Organization membership hack removed as of 2025-10-17.
    /// This field is kept for API backward compatibility but will always be false.
    /// Access now requires individual credits, trial, or subscription.
    #[deprecated(
        since = "1.0.0",
        note = "Organization membership hack removed as of 2025-10-17. Use credit-based access instead."
    )]
    pub has_organization_access: bool,
    pub access_source: crate::routes::billing::access_source::AccessSource,
}
