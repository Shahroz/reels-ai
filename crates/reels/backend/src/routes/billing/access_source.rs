//! AccessSource enum indicating the source of user access to the application.
//!
//! This enum represents how a user has gained access to the application features,
//! distinguishing between individual subscriptions/trials, organization membership,
//! both sources simultaneously, or no access. Used in billing status responses
//! to provide clear visibility into access permissions and their origins.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, serde::Serialize, utoipa::ToSchema)]
pub enum AccessSource {
    Individual,
    Organization,
    Both,
    None,
}
