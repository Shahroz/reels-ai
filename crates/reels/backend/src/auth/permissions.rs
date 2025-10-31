//! Module for reusable permission checking functions.

use crate::db::organization_members::OrganizationMember;
use crate::routes::error_response::ErrorResponse;
use actix_web::HttpResponse;
use sqlx::PgPool;
use uuid::Uuid;

/// Represents different types of permission errors.
#[derive(Debug)] pub enum PermissionError {
    DbError(()), // Field changed to () as it was unused; other variants removed as they were never constructed.
}

impl From<sqlx::Error> for PermissionError {
    fn from(_err: sqlx::Error) -> Self { // Parameter _err marked unused as its value is no longer stored.
        PermissionError::DbError(())
    }
}

/// Checks if a user is an active member of a given organization.
/// Returns Ok(OrganizationMember) if active member, or an HttpResponse error otherwise.
pub async fn check_active_membership(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<OrganizationMember, HttpResponse> {
    // 1. Check if the organization itself exists
    match crate::queries::organizations::find_organization_by_id(pool, org_id).await {
        Ok(Some(_org)) => { /* Organization exists, proceed to check membership */ }
        Ok(None) => {
            // Organization does not exist, return 404 Not Found
            return Err(HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Organization not found: {org_id}"),
            }));
        }
        Err(e) => {
            // Database error while trying to find the organization
            log::error!("DB error checking organization existence in check_active_membership: {e}. Org ID: {org_id}");
            return Err(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify organization existence.".to_string(),
            }));
        }
    }

    // 2. Organization exists, now check membership status
    match sqlx::query_as!(
        OrganizationMember,
        r#"
        SELECT organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at
        FROM organization_members
        WHERE organization_id = $1 AND user_id = $2
        "#,
        org_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(member)) => {
            if member.status != "active" {
                Err(HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied: User is not an active member of this organization.".to_string(),
                }))
            } else {
                Ok(member)
            }
        }
        Ok(None) => Err(HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied: User is not a member of this organization.".to_string(),
        })),
        Err(e) => {
            log::error!("DB error checking membership: {e}. Org ID: {org_id}, User ID: {user_id}");
            Err(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify organization membership.".to_string(),
            }))
        }
    }
}

/// Checks if a user is an active owner of a given organization.
/// Returns Ok(OrganizationMember) if active owner, or an HttpResponse error otherwise.
pub async fn check_active_owner(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<OrganizationMember, HttpResponse> {
    match check_active_membership(pool, org_id, user_id).await {
        Ok(member) => {
            if member.role != "owner" {
                Err(HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied: User must be an owner of this organization.".to_string(),
                }))
            } else {
                Ok(member)
            }
        }
        Err(resp) => Err(resp), // Forward the HttpResponse from check_active_membership
    }
}

/// Checks if a user is an active owner or admin (assuming admin role might be added later or covered by 'owner') of a given organization.
/// Returns Ok(true) if active owner/admin, Ok(false) if active but not owner/admin, or an HttpResponse error otherwise.
pub async fn check_is_org_owner_or_admin(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<bool, HttpResponse> {
    match check_active_membership(pool, org_id, user_id).await {
        Ok(member) => {
            // Assuming 'owner' is the highest privilege. If an 'admin' role is distinct and also grants this, add it here.
            if member.role == "owner" { // || member.role == "admin" (if admin role exists and grants this permission)
                Ok(true)
            } else {
                Ok(false) // Active member, but not an owner/admin
            }
        }
        Err(resp) => Err(resp), // Forward the HttpResponse from check_active_membership, which handles non-member or DB errors
    }
} 
