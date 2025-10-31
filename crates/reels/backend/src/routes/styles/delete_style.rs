//! Defines the handler for deleting a specific style resource.
//!
//! This function handles DELETE requests to `/api/styles/{id}`. It removes the
//! style identified by `id` from the database, provided it belongs to the
//! authenticated user. Returns a success (No Content) or error response.
//! Requires DB pool, path extractor, and user claims.
use tracing::instrument;

use crate::auth::tokens::Claims;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::db::shares::{AccessLevel, EntityType};
// use crate::db::styles::Style; // Not strictly needed if not accessing GCS URLs here
use crate::routes::error_response::ErrorResponse;
// use crate::services::gcs::gcs_client::GCSClient; // No longer directly used here
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

// Helper struct to fetch only user_id for permission check
#[derive(Copy, Clone, Debug)]
struct StyleOwner { user_id: Option<Uuid> }

#[utoipa::path(
    delete,
    path = "/api/styles/{id}",
    params(("id" = String, Path, description = "Style ID")),
    responses(
        (status = 204, description = "Style deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Style not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, path, claims))] // Removed gcs_client
pub async fn delete_style(
    pool: web::Data<PgPool>,
    // gcs_client: web::Data<GCSClient>, // Removed
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let style_id = path.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction for style deletion: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Database error."));
        }
    };

    let style_owner_result = sqlx::query_as!( 
        StyleOwner, 
        "SELECT user_id FROM styles WHERE id = $1", style_id
    )
    .fetch_optional(&mut *tx)
    .await;

    let can_delete = match style_owner_result {
        Ok(Some(details)) => {
            if details.user_id == Some(authenticated_user_id) {
                true
            } else if details.user_id.is_none() {
                // Public styles can only be deleted by admins
                claims.is_admin
            } else {
                let org_memberships = match find_active_memberships_for_user(&pool, authenticated_user_id).await {
                    Ok(memberships) => memberships,
                    Err(e) => {
                        log::error!("Error fetching org memberships (delete_style) for user {authenticated_user_id}: {e}");
                        if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
                        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check permissions."));
                    }
                };
                let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
                let org_ids_slice: &[Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };
                let sql_object_type = "style".to_string();

                match sqlx::query_scalar!(
                    r#"
                        SELECT EXISTS (
                            SELECT 1 FROM object_shares
                            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
                            AND (
                                (entity_type = $4 AND entity_id = $5) OR
                                (entity_type = $6 AND entity_id = ANY($7))
                            )
                        )
                    "#,
                    style_id, sql_object_type, AccessLevel::Editor as AccessLevel, 
                    EntityType::User as EntityType, authenticated_user_id, 
                    EntityType::Organization as EntityType, org_ids_slice
                )
                .fetch_one(&mut *tx)
                .await {
                    Ok(Some(true)) => true,
                    Ok(Some(false)) | Ok(None) => false, 
                    Err(e) => {
                        log::error!("Error checking 'editor' share for style deletion {style_id}: {e}");
                        if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
                        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify permissions."));
                    }
                }
            }
        }
        Ok(None) => false, // Style not found, so can_delete is false
        Err(e) => {
            log::error!("Error fetching style for delete permission check {style_id}: {e}");
            if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve style for deletion check."));
        }
    };

    if !can_delete {
        if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
        // Log if style existed but no permission, or if style didn't exist at all
        if style_owner_result.as_ref().ok().is_some_and(|o| o.is_some()) {
            log::warn!("User {authenticated_user_id} lacks permission to delete style {style_id}.");
        } else {
            log::info!("Attempt to delete non-existent style {style_id} by user {authenticated_user_id}.");
        }
        return HttpResponse::NotFound().json(ErrorResponse::from("Style not found or access denied."));
    }
    
    // TODO: Add GCS object deletion here for the style's html_url and screenshot_url.
    //       This should ideally be done before the DB transaction commits, or be an idempotent operation.
    //       If GCS deletion fails critically, the transaction might need to be rolled back.
    //       Example: Fetch GCS URLs from `styles` table (if not already available) before deleting the record.

    let sql_object_type_for_delete = "style";
    let shares_delete_result = sqlx::query!(
        "DELETE FROM object_shares WHERE object_id = $1 AND object_type = $2",
        style_id,
        sql_object_type_for_delete
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = shares_delete_result {
        log::error!("Failed to delete shares for style {style_id}: {e}");
        if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to clean up style shares."));
    }

    let style_delete_result = sqlx::query!("DELETE FROM styles WHERE id = $1", style_id)
        .execute(&mut *tx)
        .await;

    match style_delete_result {
        Ok(res) if res.rows_affected() > 0 => {
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction for style deletion: {e}");
                return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize style deletion."));
            }
            HttpResponse::NoContent().finish()
        }
        Ok(_) => { 
            if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
            log::warn!("Style {style_id} not found during final DB delete operation (was permission check based on outdated info?).");
            HttpResponse::NotFound().json(ErrorResponse::from("Style not found during deletion."))
        }
        Err(e) => {
            if let Err(rb_err) = tx.rollback().await { log::error!("Failed to rollback transaction: {rb_err}");}
            log::error!("Error deleting style record {style_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to delete style from database."))
        }
    }
}
