//! Handler for deleting an object share.
use crate::auth::tokens::Claims;
use crate::db::shares::{AccessLevel, EntityType, ObjectShare};
use crate::routes::error_response::ErrorResponse;
use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

// Helper to check if user can manage shares for an object (owner or editor)
// This is identical to the one in create_share.rs and list_shares.rs - should be refactored to a common place.
async fn can_user_manage_object_shares(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, HttpResponse> {
    let is_owner = match object_type {
        "style" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking style ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        "creative" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM creatives c JOIN collections col ON c.collection_id = col.id WHERE c.id = $1 AND col.user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking creative ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        "document" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking document ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        // "research" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM research WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking research ownership: {}", e); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        // "research_workflow" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM research_workflows WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking research_workflow ownership: {}", e); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        "custom_format" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM custom_creative_formats WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking custom_format ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        "asset" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking asset ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        "collection" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking collection ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to check ownership".to_string() }) })?.unwrap_or(false),
        _ => return Err(HttpResponse::BadRequest().json(ErrorResponse{error: "Unsupported object type for share deletion.".to_string()})),
    };

    if is_owner {
        return Ok(true);
    }

    let org_memberships = crate::queries::organizations::find_active_memberships_for_user(pool, user_id).await.map_err(|e|{
        log::error!("DB error fetching org memberships for share management: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to verify permissions".to_string()})
    })?;
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice: &[Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

    let has_editor_share = sqlx::query_scalar!(
        r#"SELECT EXISTS (
            SELECT 1 FROM object_shares
            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
            AND (
                (entity_type = 'user' AND entity_id = $4) OR 
                (entity_type = 'organization' AND entity_id = ANY($5))
            )
        )"#, 
        object_id, 
        object_type,
        AccessLevel::Editor as AccessLevel,
        user_id,
        org_ids_slice
    )
    .fetch_one(pool).await.map_err(|e| {
        log::error!("DB error checking editor share: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to verify share permissions".to_string()})
    })?.unwrap_or(false);

    Ok(has_editor_share)
}

#[utoipa::path(
    delete,
    path = "/api/shares/{share_id}",
    params(
        ("share_id" = Uuid, Path, description = "ID of the share to delete")
    ),
    responses(
        (status = 204, description = "Share deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - User cannot delete this share", body = ErrorResponse),
        (status = 404, description = "Share not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Shares",
    security(("user_auth" = []))
)]
#[delete("/{share_id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_share(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    share_id: web::Path<Uuid>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let share_id_to_delete = share_id.into_inner();

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            log::error!("Failed to start transaction for deleting share: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse{error: "Database error".to_string()});
        }
    };

    // 1. Fetch the share to get object_id and object_type for permission check
    let share_to_delete = match sqlx::query_as!(ObjectShare, 
        r#"SELECT 
            id, object_id, object_type, entity_id, entity_type AS "entity_type!: EntityType", 
            access_level AS "access_level!: AccessLevel", created_at, updated_at,
            NULL::text AS entity_name
           FROM object_shares WHERE id = $1"#,
           share_id_to_delete
        )
        .fetch_optional(&mut *tx)
        .await {
        Ok(Some(share)) => share,
        Ok(None) => {
            if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error after share not found: {rb_err}"); }
            return HttpResponse::NotFound().json(ErrorResponse{error: "Share not found.".to_string()});
        }
        Err(e) => {
            if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error after failing to fetch share: {rb_err}"); }
            log::error!("DB error fetching share for deletion: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to retrieve share details.".to_string()});
        }
    };

    // 2. Permission Check:
    // User must own the object OR have an 'editor' share on the object 
    // OR be the entity to whom the share was granted (they can remove their own access).
    let can_manage = match can_user_manage_object_shares(&pool, authenticated_user_id, share_to_delete.object_id, &share_to_delete.object_type).await {
        Ok(val) => val,
        Err(resp) => { 
            if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error after permission check failed: {rb_err}"); }
            return resp; 
        }
    };

    let is_recipient_of_share = share_to_delete.entity_type == EntityType::User && share_to_delete.entity_id == authenticated_user_id;
    // TODO: Add logic if entity_type is Organization and user is part of that org, should they be able to remove the org's share? 
    // For now, only direct user recipients or object managers can delete.

    if !can_manage && !is_recipient_of_share {
        if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error on forbidden deletion: {rb_err}"); }
        log::warn!("User {} forbidden to delete share {} (object_id: {}, object_type: {}). User is not manager and not direct recipient.", authenticated_user_id, share_id_to_delete, share_to_delete.object_id, share_to_delete.object_type);
        return HttpResponse::Forbidden().json(ErrorResponse{error: "You do not have permission to delete this share.".to_string()});
    }

    // 3. Delete the share
    match sqlx::query!("DELETE FROM object_shares WHERE id = $1", share_id_to_delete)
        .execute(&mut *tx)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // This case should ideally be caught by the fetch_optional above, but as a safeguard:
                if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error, share to delete not found during delete op: {rb_err}"); }
                return HttpResponse::NotFound().json(ErrorResponse{error: "Share not found during deletion.".to_string()});
            }
            // Commit transaction
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction for deleting share: {e}");
                return HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to finalize share deletion.".to_string()});
            }
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            if let Err(rb_err) = tx.rollback().await { log::error!("Rollback error after failing to delete share: {rb_err}"); }
            log::error!("DB error deleting share: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse{error: "Failed to delete share.".to_string()})
        }
    }
} 
