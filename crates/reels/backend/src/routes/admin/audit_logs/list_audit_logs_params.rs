//! Defines query parameters for the list audit logs endpoint.
//!
//! This struct specifies all available filters and pagination options for querying audit logs.
//! All parameters are optional, allowing flexible filtering for admin investigations.
//! Supports filtering by admin user, action type, entity type, target entity, and date range.

#[derive(Debug, serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Clone)]
pub struct ListAuditLogsParams {
    /// Page number for pagination (starts at 1).
    #[param(default = 1)]
    pub page: Option<i64>,
    
    /// Number of items per page.
    #[param(default = 20)]
    pub limit: Option<i64>,
    
    /// Filter by the admin user who performed the action.
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub admin_user_id: Option<uuid::Uuid>,
    
    /// Filter by action type (e.g., "CREATE_ORGANIZATION", "DELETE_USER").
    #[param(example = "CREATE_ORGANIZATION")]
    pub action_type: Option<String>,
    
    /// Filter by target entity type (e.g., "Organization", "User").
    #[param(example = "Organization")]
    pub target_entity_type: Option<String>,
    
    /// Filter by target entity ID.
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub target_entity_id: Option<uuid::Uuid>,
    
    /// Filter by logs created after this date (ISO 8601 format).
    #[param(example = "2025-10-01T00:00:00Z")]
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by logs created before this date (ISO 8601 format).
    #[param(example = "2025-10-10T23:59:59Z")]
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

