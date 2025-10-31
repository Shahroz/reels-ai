//! Defines query parameters for the list all organizations endpoint.
//!
//! This struct specifies pagination, search, and sorting options for the admin
//! organizations list endpoint. Supports flexible filtering and sorting for admin
//! interfaces managing all organizations in the system.

#[derive(Debug, serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Clone)]
pub struct ListAllOrganizationsParams {
    /// Page number for pagination (starts at 1).
    #[param(default = 1)]
    pub page: Option<i64>,

    /// Number of items per page.
    #[param(default = 20)]
    pub limit: Option<i64>,

    /// Search term to filter organizations by name or owner email.
    #[param(example = "acme")]
    pub search: Option<String>,

    /// Field to sort by: "name", "created_at", or "owner_email".
    #[param(default = "created_at", example = "name")]
    pub sort_by: Option<String>,

    /// Sort order: "asc" or "desc".
    #[param(default = "desc", example = "asc")]
    pub sort_order: Option<String>,
}
