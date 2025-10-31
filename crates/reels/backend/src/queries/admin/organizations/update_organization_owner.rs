//! Updates the owner of an organization and adjusts member roles.
//!
//! This query transfers organization ownership from one user to another.
//! It updates the organization's owner_user_id, demotes the old owner to member role,
//! and promotes the new owner to owner role (adding them as a member if needed).
//! Must be called within a transaction to ensure atomicity.

pub async fn update_organization_owner(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    organization_id: uuid::Uuid,
    old_owner_id: uuid::Uuid,
    new_owner_id: uuid::Uuid,
) -> anyhow::Result<crate::db::organizations::Organization> {
    let updated_org = sqlx::query_as!(
        crate::db::organizations::Organization,
        r#"
        UPDATE organizations
        SET owner_user_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        "#,
        organization_id,
        new_owner_id
    )
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE organization_members
        SET role = 'member'
        WHERE organization_id = $1 AND user_id = $2
        "#,
        organization_id,
        old_owner_id
    )
    .execute(&mut **tx)
    .await?;

    let now = chrono::Utc::now();
    sqlx::query!(
        r#"
        INSERT INTO organization_members (organization_id, user_id, role, status, invited_at, joined_at)
        VALUES ($1, $2, 'owner', 'active', $3, $3)
        ON CONFLICT (organization_id, user_id) 
        DO UPDATE SET role = 'owner', status = 'active', joined_at = COALESCE(organization_members.joined_at, $3)
        "#,
        organization_id,
        new_owner_id,
        now
    )
    .execute(&mut **tx)
    .await?;

    Ok(updated_org)
}
