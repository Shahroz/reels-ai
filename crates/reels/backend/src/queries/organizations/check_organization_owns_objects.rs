//! Checks if an organization owns any objects across various tables.
use sqlx::{types::Uuid, PgPool};

/// Checks if an organization owns any objects across various tables.
/// Returns true if any objects are found, false otherwise.
/// This function needs to list all tables that can have an organization_id.
#[allow(dead_code)] // TODO: Integrate this check into the organization deletion workflow before calling delete_organization_by_id
pub async fn check_organization_owns_objects(pool: &PgPool, org_id: Uuid) -> anyhow::Result<bool> {
    // Collections are user-only and should not have an organization_id.
    // // Collections
    // let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM collections WHERE organization_id = $1")
    //     .bind(org_id)
    //     .fetch_one(pool)
    //     .await?;
    // if count > 0 { return Ok(true); }

    // Styles
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM styles WHERE organization_id = $1")
        .bind(org_id)
        .fetch_one(pool)
        .await?;
    if count > 0 { return Ok(true); }

    // Creatives
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM creatives WHERE organization_id = $1")
        .bind(org_id)
        .fetch_one(pool)
        .await?;
    if count > 0 { return Ok(true); }

    // Assets
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE organization_id = $1")
        .bind(org_id)
        .fetch_one(pool)
        .await?;
    if count > 0 { return Ok(true); }

    // Research Workflows
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM research_workflows WHERE organization_id = $1")
        .bind(org_id)
        .fetch_one(pool)
        .await?;
    if count > 0 { return Ok(true); }

    // Add checks for any other tables that might link to organizations here.

    Ok(false) // No objects found linked to this organization
} 