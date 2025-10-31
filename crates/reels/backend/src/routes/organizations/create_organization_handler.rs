//! Handles the creation of a new organization.
//!
//! This endpoint receives the organization details in the request body,
//! uses the authenticated user's ID as the owner, creates the organization
//! in the database, and returns the newly created organization data.
//! Follows standard API patterns and error handling.

use crate::auth::tokens::Claims;
use crate::db::organizations::Organization;
use crate::queries::organizations::{add_member, create_organization};
use crate::routes::error_response::ErrorResponse;
use crate::routes::organizations::create_organization_request::CreateOrganizationRequest;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa; // Added for macro use

/// Creates a new organization.
///
/// Creates an organization with the provided name, setting the authenticated user as the owner.
#[utoipa::path(
    post,
    path = "/api/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = Organization),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[post("")]
pub async fn create_organization_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    payload: web::Json<CreateOrganizationRequest>,
) -> impl Responder {
    let user_id = claims.user_id; // Get user ID from JWT claims

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create organization".to_string(),
            });
        }
    };

    match create_organization(&mut tx, &payload.name, user_id).await {
        Ok(organization) => {
            // Add the creator as an owner member
            match add_member(
                &mut tx, // Use the transaction
                organization.id,
                user_id,
                "owner",  // Role
                "active", // Status
                None,     // invited_by_user_id (owner is not invited)
            )
            .await
            {
                Ok(_) => {
                    // Commit the transaction
                    if let Err(e) = tx.commit().await {
                        log::error!("Failed to commit transaction: {e}");
                        return HttpResponse::InternalServerError().json(ErrorResponse {
                            error: "Failed to create organization".to_string(),
                        });
                    }
                    HttpResponse::Created().json(organization)
                }
                Err(e) => {
                    log::error!("Failed to add owner as member: {e}");
                    // Attempt to rollback
                    if let Err(rb_err) = tx.rollback().await {
                        log::error!("Failed to rollback transaction: {rb_err}");
                    }
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to create organization".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create organization in DB: {e}");
            // Attempt to rollback (though create_organization might not have started a tx itself, this is for the outer tx)
            if let Err(rb_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {rb_err}");
            }
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create organization".to_string(),
            })
        }
    }
}