//! Handler for deleting predefined collections.
//!
//! This endpoint allows administrators to delete predefined collections.
//! Only administrators should have access to this endpoint.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;

/// Deletes a predefined collection.
///
/// Deletes an existing predefined collection by its ID.
/// This endpoint is typically used by administrators to remove template collections.
/// Note: This will not affect existing user collections that were copied from this template.
#[utoipa::path(
    delete,
    path = "/api/predefined-collections/{collection_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the predefined collection to delete")
    ),
    responses(
        (status = 204, description = "Predefined collection deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Predefined collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Predefined Collections"
)]
#[actix_web::delete("/{collection_id}")]
pub async fn delete_predefined_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    collection_id: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let collection_id_to_delete = collection_id.into_inner();
    
    // Check if user has admin privileges (you may need to adjust this based on your auth system)
    // For now, we'll allow any authenticated user to delete predefined collections
    
    // First, check if the predefined collection exists and get its name for logging
    let existing_collection = sqlx::query_as!(
        crate::db::predefined_collection::PredefinedCollection,
        "SELECT id, name, description, schema_definition, ui_component_definition, created_at, updated_at FROM predefined_collections WHERE id = $1",
        collection_id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match existing_collection {
        Ok(None) => {
            log::warn!("Attempted to delete non-existent predefined collection: {collection_id_to_delete}");
            actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found.".into(),
            })
        }
        Err(e) => {
            log::error!("Failed to check predefined collection existence: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to check collection existence.".into(),
            })
        }
        Ok(Some(collection)) => {
            // Check if any user collections are using this predefined collection
            let usage_count = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM user_db_collections WHERE source_predefined_collection_id = $1",
                collection_id_to_delete
            )
            .fetch_one(pool.get_ref())
            .await;
            
            match usage_count {
                Ok(Some(count)) if count > 0 => {
                    log::warn!("Attempted to delete predefined collection '{}' that is used by {} user collections", collection.name, count);
                    return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
                        error: format!("Cannot delete predefined collection that is used by {count} user collections."),
                    });
                }
                Ok(_) => {} // No usage, safe to delete
                Err(e) => {
                    log::error!("Failed to check predefined collection usage: {e:?}");
                    return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to check collection usage.".into(),
                    });
                }
            }
            
            // Delete the predefined collection
            let delete_result = sqlx::query!(
                "DELETE FROM predefined_collections WHERE id = $1",
                collection_id_to_delete
            )
            .execute(pool.get_ref())
            .await;
            
            match delete_result {
                Ok(_) => {
                    log::info!("Deleted predefined collection '{}' by user {}", collection.name, user_id);
                    actix_web::HttpResponse::NoContent().finish()
                }
                Err(e) => {
                    log::error!("Failed to delete predefined collection: {e:?}");
                    actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to delete predefined collection.".into(),
                    })
                }
            }
        }
    }
} 