//! Defines the response structure for the list audit logs endpoint.
//!
//! This struct wraps the paginated list of audit logs with total count metadata.
//! Used by admin interfaces to display audit trails with proper pagination controls.

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListAuditLogsResponse {
    /// A list of audit log entries.
    pub items: Vec<crate::db::audit_logs::AuditLog>,
    
    /// The total number of audit logs matching the query filters.
    pub total_count: i64,
    
    /// The current page number.
    pub page: i64,
    
    /// The number of items per page.
    pub limit: i64,
}

