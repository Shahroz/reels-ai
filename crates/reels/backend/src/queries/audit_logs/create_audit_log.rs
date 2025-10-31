//! Creates a new audit log entry in the database.
//!
//! This function records administrative actions for compliance and debugging purposes.
//! It can be called within transactions or standalone, accepting any SQLx executor.
//! Returns the created audit log entry with all fields populated including the generated ID.
//! Designed for use across all admin operations to maintain a complete audit trail.
//! Accepts type-safe AuditAction enum to ensure consistency.

pub async fn create_audit_log(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    admin_user_id: uuid::Uuid,
    action: crate::db::audit_action::AuditAction,
    target_entity_type: &str,
    target_entity_id: Option<uuid::Uuid>,
    metadata: Option<serde_json::Value>,
) -> anyhow::Result<crate::db::audit_logs::AuditLog> {
    let action_type = action.as_str();
    let log = sqlx::query_as!(
        crate::db::audit_logs::AuditLog,
        r#"
        INSERT INTO audit_logs (admin_user_id, action_type, target_entity_type, target_entity_id, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, admin_user_id, action_type, target_entity_type, target_entity_id, metadata, created_at
        "#,
        admin_user_id,
        action_type,
        target_entity_type,
        target_entity_id,
        metadata
    )
    .fetch_one(executor)
    .await?;
    
    Ok(log)
}

