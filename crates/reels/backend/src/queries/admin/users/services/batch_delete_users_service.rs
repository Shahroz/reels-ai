//! Service for batch deleting users with audit logging.
//!
//! This service validates and deletes multiple user accounts with safety checks
//! to prevent admins from deleting themselves or other admin users. The batch
//! operations are performed individually and the audit log is created in a
//! transaction to ensure it's recorded.

pub async fn batch_delete_users_service(
    pool: &sqlx::PgPool,
    admin_user_id: uuid::Uuid,
    user_ids: Vec<uuid::Uuid>,
) -> anyhow::Result<crate::queries::admin::users::batch_delete_users::BatchDeleteUsersResult> {
    if user_ids.is_empty() {
        return Err(anyhow::anyhow!("At least one user ID must be provided"));
    }

    let result = crate::queries::admin::users::batch_delete_users(
        pool,
        user_ids,
        admin_user_id,
    )
    .await?;

    let mut tx = pool.begin().await?;

    let deleted_user_ids: Vec<String> = result
        .success
        .iter()
        .map(|s| s.user_id.to_string())
        .collect();

    let metadata = serde_json::json!({
        "success_count": result.success.len(),
        "failed_count": result.failed.len(),
        "deleted_user_ids": deleted_user_ids,
    });

    crate::queries::audit_logs::create_audit_log(
        &mut *tx,
        admin_user_id,
        crate::db::audit_action::AuditAction::DeleteUsersBatch,
        "User",
        None,
        Some(metadata),
    )
    .await?;

    tx.commit().await?;
    Ok(result)
}

