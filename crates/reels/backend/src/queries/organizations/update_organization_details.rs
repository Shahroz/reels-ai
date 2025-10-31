//! Updates an organization's details, e.g., its name.
use crate::db::organizations::Organization;
use crate::queries::organizations::find_organization_by_id;
use sqlx::{types::Uuid, PgPool};

/// Updates an organization's details, e.g., its name.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `org_id` - The UUID of the organization to update.
/// * `name` - The new name for the organization. If None, name is not updated.
///
/// # Returns
///
/// A `Result` containing the updated `Organization` on success, or an `sqlx::Error` if the org is not found or on other DB errors.
pub async fn update_organization_details(
    pool: &PgPool,
    org_id: Uuid,
    name: Option<String>,
    // Add settings: Option<serde_json::Value> here later if needed
) -> anyhow::Result<Option<Organization>> { // Return Option<Organization> to indicate if update occurred
    if name.is_none() /* && settings.is_none() etc. */ {
        // If no fields are provided for update, fetch and return the current organization
        // or return an error/specific response indicating no update was performed.
        // For now, let's return Ok(None) to signify no update action was taken for name.
        // Better might be to fetch the org and return it, or expect at least one field.
        // For simplicity here, if name is None, we won't update.
        // A more robust solution would build the query dynamically or have specific update functions.
        return find_organization_by_id(pool, org_id).await; // Return current if no name change
    }

    // Only update if name is Some. This query only updates name.
    // A more complex version could handle multiple optional fields.
    let updated_org = sqlx::query_as!(
        Organization,
        r#"
        UPDATE organizations
        SET name = COALESCE($1, name), updated_at = NOW()
        WHERE id = $2
        RETURNING id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        "#,
        name,
        org_id
    )
    .fetch_optional(pool) // Use fetch_optional as update might not find the row
    .await?;

    Ok(updated_org)
} 