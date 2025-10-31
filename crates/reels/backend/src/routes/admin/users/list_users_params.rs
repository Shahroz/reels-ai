//! Defines query parameters for the list users endpoint.

use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for listing users in the admin panel. Also used for usage statistics.
#[derive(Debug, Deserialize, IntoParams, ToSchema, Clone)]
pub struct ListUsersParams {
    /// Page number for pagination (starts at 1).
    #[param(default = 1)]
    pub page: Option<i64>,
    /// Number of items per page.
    #[param(default = 10)]
    pub limit: Option<i64>,
    /// Field to sort by. Valid options: "email", "created_at", "status".
    #[param(default = "created_at")]
    pub sort_by: Option<String>,
    /// Sort order. Valid options: "asc", "desc".
    #[param(default = "desc")]
    pub sort_order: Option<String>,
    /// Search users by email or other fields (case-insensitive, partial match).
    pub search: Option<String>,
    /// Filter users by status.
    pub status: Option<String>,
}
