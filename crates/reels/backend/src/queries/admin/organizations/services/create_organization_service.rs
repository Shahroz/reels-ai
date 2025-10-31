//! Service for creating an organization with audit logging.
//!
//! This service encapsulates the complete business operation: creating an organization,
//! adding the owner as a member, and logging the action to the audit trail. All operations
//! are performed within a single transaction to ensure atomicity. The audit log is mandatory
//! and cannot be skipped, ensuring complete traceability of all admin actions.

pub async fn create_organization_service(
    pool: &sqlx::PgPool,
    admin_user_id: uuid::Uuid,
    name: &str,
    owner_user_id: uuid::Uuid,
) -> Result<crate::db::organizations::Organization, crate::queries::admin::admin_service_error::AdminServiceError> {
    if name.trim().is_empty() {
        return Err(crate::queries::admin::admin_service_error::AdminServiceError::EmptyOrganizationName);
    }
    
    if !crate::queries::users::user_exists(pool, owner_user_id).await? {
        return Err(crate::queries::admin::admin_service_error::AdminServiceError::OwnerUserNotFound);
    }
    
    let mut tx = pool.begin().await?;
    
    let org = crate::queries::organizations::create_organization::create_organization(
        &mut tx,
        name,
        owner_user_id,
    )
    .await?;
    
    crate::queries::organizations::add_member::add_member(
        &mut tx,
        org.id,
        owner_user_id,
        "owner",
        "active",
        Some(admin_user_id),
    )
    .await?;
    
    let metadata = serde_json::json!({
        "organization_name": org.name,
        "owner_user_id": owner_user_id.to_string(),
    });
    
    crate::queries::audit_logs::create_audit_log(
        &mut *tx,
        admin_user_id,
        crate::db::audit_action::AuditAction::CreateOrganization,
        "Organization",
        Some(org.id),
        Some(metadata),
    )
    .await?;
    
    tx.commit().await?;
    
    Ok(org)
}

