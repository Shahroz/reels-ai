//! Retrieves a single audit log entry by its ID.
//!
//! Returns the complete audit log record if found, or an error if not found.
//! Useful for detailed inspection of specific administrative actions.
//! Used primarily by admin interfaces for drill-down views and investigations.

pub async fn get_audit_log_by_id(
    pool: &sqlx::PgPool,
    log_id: uuid::Uuid,
) -> anyhow::Result<crate::db::audit_logs::AuditLog> {
    let log = sqlx::query_as!(
        crate::db::audit_logs::AuditLog,
        r#"
        SELECT id, admin_user_id, action_type, target_entity_type, target_entity_id, metadata, created_at
        FROM audit_logs
        WHERE id = $1
        "#,
        log_id
    )
    .fetch_one(pool)
    .await?;

    Ok(log)
}

