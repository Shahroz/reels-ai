//! Service for batch creating users with audit logging.
//!
//! This service validates and creates multiple user accounts, automatically logging
//! the batch operation to the audit trail. The batch operations are performed  
//! individually and the audit log is created in a transaction to ensure it's recorded.
//! The service provides detailed success/failure results for each email address.

pub async fn batch_create_users_service(
    pool: &sqlx::PgPool,
    admin_user_id: uuid::Uuid,
    emails: Vec<String>,
) -> anyhow::Result<crate::queries::admin::users::batch_create_users_result::BatchCreateUsersResult> {
    if emails.is_empty() {
        return Err(anyhow::anyhow!("At least one email must be provided"));
    }

    let result = crate::queries::admin::users::batch_create_users(pool, emails).await?;

    let mut tx = pool.begin().await?;

    let success_emails: Vec<String> = result
        .success
        .iter()
        .map(|s| s.email.clone())
        .collect();

    let metadata = serde_json::json!({
        "success_count": result.success.len(),
        "failed_count": result.failed.len(),
        "success_emails": success_emails,
    });

    crate::queries::audit_logs::create_audit_log(
        &mut *tx,
        admin_user_id,
        crate::db::audit_action::AuditAction::CreateUsersBatch,
        "User",
        None,
        Some(metadata),
    )
    .await?;

    tx.commit().await?;
    Ok(result)
}

