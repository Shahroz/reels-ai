//! Represents an audit log entry in the database.
//!
//! This file defines the `AuditLog` struct, mirroring the `audit_logs` table schema.
//! Audit logs track all administrative actions for security, compliance, and debugging.
//! Query functions are located in `crate::queries::audit_logs`.
//! Adheres to the project's Rust coding standards and one-file-per-item pattern.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuditLog {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub admin_user_id: uuid::Uuid,
    
    #[schema(example = "CREATE_ORGANIZATION")]
    pub action_type: String,
    
    #[schema(example = "Organization")]
    pub target_entity_type: String,
    
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub target_entity_id: Option<uuid::Uuid>,
    
    #[schema(value_type = Option<Object>, example = json!({"organization_name": "NewCo"}))]
    pub metadata: Option<serde_json::Value>,
    
    #[schema(value_type = String, format = "date-time", example = "2025-10-10T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

