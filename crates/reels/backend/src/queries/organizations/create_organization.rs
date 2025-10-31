//! Creates a new organization in the database.
use crate::db::organizations::Organization;
use sqlx::{types::Uuid, Transaction, Postgres};

/// Creates a new organization in the database.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `name` - The name of the new organization.
/// * `owner_user_id` - The UUID of the user who owns this organization.
///
/// # Returns
///
/// A `Result` containing the newly created `Organization` on success, or an `sqlx::Error` on failure.
pub async fn create_organization(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    owner_user_id: Uuid,
) -> anyhow::Result<Organization> {
    let org = sqlx::query_as!(
        Organization,
        r#"
        INSERT INTO organizations (name, owner_user_id, is_personal)
        VALUES ($1, $2, false)
        RETURNING id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        "#,
        name,
        owner_user_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(org)
} 