//! Service for updating an organization with audit logging.
//!
//! This service handles updating organization name and/or transferring ownership.
//! All changes are performed within a single transaction and automatically logged
//! to the audit trail. Each type of update (name change, owner transfer) creates
//! a separate audit log entry for granular tracking.
//!
//! When ownership is transferred and the organization has a Stripe customer,
//! the customer's email is updated to the new owner's email.

use crate::services::billing::billing_factory::create_billing_service;
use crate::services::billing::billing_config::BillingConfig;

pub async fn update_organization_service(
    pool: &sqlx::PgPool,
    admin_user_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    new_name: Option<String>,
    new_owner_user_id: Option<uuid::Uuid>,
) -> anyhow::Result<crate::db::organizations::Organization> {
    if new_name.is_none() && new_owner_user_id.is_none() {
        return Err(anyhow::anyhow!(
            "At least one field must be provided for update"
        ));
    }

    if let Some(ref name) = new_name {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Organization name cannot be empty"));
        }
    }

    let mut tx = pool.begin().await?;

    let existing_org = sqlx::query_as!(
        crate::db::organizations::Organization,
        r#"SELECT id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at FROM organizations WHERE id = $1"#,
        organization_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Organization not found"))?;

    let mut updated_org = existing_org.clone();

    if let Some(ref new_name_value) = new_name {
        let row = sqlx::query!(
            r#"UPDATE organizations SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING name, updated_at"#,
            new_name_value,
            organization_id
        )
        .fetch_one(&mut *tx)
        .await?;

        updated_org.name = row.name;
        updated_org.updated_at = row.updated_at;

        let metadata = serde_json::json!({
            "old_name": existing_org.name,
            "new_name": new_name_value,
        });

        crate::queries::audit_logs::create_audit_log(
            &mut *tx,
            admin_user_id,
            crate::db::audit_action::AuditAction::UpdateOrganizationName,
            "Organization",
            Some(organization_id),
            Some(metadata),
        )
        .await?;
    }

    // Track if ownership was transferred for Stripe update after commit
    let mut ownership_transferred = false;
    let mut new_owner_email: Option<String> = None;

    if let Some(new_owner_id) = new_owner_user_id {
        if new_owner_id == existing_org.owner_user_id {
            return Err(anyhow::anyhow!(
                "New owner is already the current owner"
            ));
        }

        if !crate::queries::users::user_exists(&mut *tx, new_owner_id).await? {
            return Err(anyhow::anyhow!("New owner user does not exist"));
        }

        // Get new owner's email before transferring
        let new_owner = sqlx::query!(
            r#"SELECT email FROM users WHERE id = $1"#,
            new_owner_id
        )
        .fetch_one(&mut *tx)
        .await?;

        new_owner_email = Some(new_owner.email);

        updated_org = crate::queries::admin::organizations::update_organization_owner(
            &mut tx,
            organization_id,
            existing_org.owner_user_id,
            new_owner_id,
        )
        .await?;

        ownership_transferred = true;

        let metadata = serde_json::json!({
            "old_owner_user_id": existing_org.owner_user_id.to_string(),
            "new_owner_user_id": new_owner_id.to_string(),
        });

        crate::queries::audit_logs::create_audit_log(
            &mut *tx,
            admin_user_id,
            crate::db::audit_action::AuditAction::ChangeOrganizationOwner,
            "Organization",
            Some(organization_id),
            Some(metadata),
        )
        .await?;
    }

    // Commit transaction BEFORE updating Stripe
    tx.commit().await?;

    // Update Stripe customer email if ownership was transferred and org has Stripe customer
    if ownership_transferred {
        if let Some(stripe_customer_id) = &updated_org.stripe_customer_id {
            if let Some(email) = new_owner_email {
                log::info!(
                    "Updating Stripe customer {} email to new owner: {}",
                    stripe_customer_id,
                    email
                );

                // Create billing service
                let config = BillingConfig::from_env();
                let billing_service = match create_billing_service(&config) {
                    Ok(service) => service,
                    Err(e) => {
                        log::error!(
                            "Failed to create billing service for Stripe email update: {}. \
                            Ownership transfer succeeded, but Stripe email was not updated.",
                            e
                        );
                        return Ok(updated_org);
                    }
                };

                // Update customer email in Stripe
                match billing_service.update_customer_email(stripe_customer_id, &email).await {
                    Ok(_) => {
                        log::info!(
                            "Successfully updated Stripe customer {} email for organization ownership transfer",
                            stripe_customer_id
                        );
                    }
                    Err(e) => {
                        // Log error but don't fail the ownership transfer
                        // The database change is already committed
                        log::error!(
                            "Failed to update Stripe customer {} email after ownership transfer: {}. \
                            Database ownership was transferred successfully, but Stripe email update failed. \
                            The new owner can update their billing email manually.",
                            stripe_customer_id,
                            e
                        );
                        // Don't return error - ownership transfer succeeded
                    }
                }
            }
        } else {
            log::info!(
                "Organization {} has no Stripe customer yet (no purchases made), skipping Stripe email update",
                organization_id
            );
        }
    }

    Ok(updated_org)
}

