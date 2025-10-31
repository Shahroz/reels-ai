//! Handler for deleting a vocal tour and its associated assets.
//!
//! Defines the `delete_vocal_tour` HTTP handler under `/api/vocal-tour/{id}`.
//! This process is destructive and will remove the vocal tour record and all
//! linked assets from both the database and GCS.

use tracing::instrument;

#[utoipa::path(
    delete,
    path = "/api/vocal-tour/{id}",
    tag = "Vocal Tour",
    params(
        ("id" = uuid::Uuid, Path, description = "Vocal Tour ID")
    ),
    responses(
        (status = 204, description = "No Content. Vocal tour and associated assets deleted."),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Vocal tour not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, gcs_client, path, claims))]
pub async fn delete_vocal_tour(
    pool: actix_web::web::Data<sqlx::PgPool>,
    gcs_client: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let authenticated_user_id = claims.user_id;
    let vocal_tour_id = path.into_inner();

    // 1. Fetch vocal tour to check ownership and get asset IDs
    let vocal_tour = match crate::queries::vocal_tours::get_vocal_tour_by_id::get_vocal_tour_by_id(&pool, vocal_tour_id).await {
        Ok(Some(tour)) => tour,
        Ok(None) => return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse::from("Vocal tour not found.")),
        Err(e) => {
            tracing::error!("Failed to fetch vocal tour {}: {}", vocal_tour_id, e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to retrieve vocal tour for deletion."));
        }
    };

    // 2. Permission Check
    if vocal_tour.user_id != authenticated_user_id {
        tracing::warn!(
            "User {} attempted to delete vocal tour {} owned by user {} (permission denied).",
            authenticated_user_id, vocal_tour_id, vocal_tour.user_id
        );
        return actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse::from("You do not have permission to delete this vocal tour."));
    }

    // 3. Fetch all associated assets to get their GCS object names
    let mut assets_to_delete = std::vec::Vec::new();
    if !vocal_tour.asset_ids.is_empty() {
        for asset_id in &vocal_tour.asset_ids {
            match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, *asset_id).await {
                Ok(Some(asset)) => assets_to_delete.push(asset),
                Ok(None) => tracing::warn!("Asset ID {} found in vocal tour {} but asset record not found in DB.", asset_id, vocal_tour_id),
                Err(e) => {
                    tracing::error!("Failed to fetch asset {} for deletion: {}. Aborting vocal tour deletion.", asset_id, e);
                    return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to retrieve associated asset for deletion."));
                }
            }
        }
    }

    // 4. Delete vocal tour from DB first. If this fails, we abort.
    match crate::queries::vocal_tours::delete_vocal_tour::delete_vocal_tour(&pool, vocal_tour_id).await {
        Ok(_) => tracing::info!("Successfully deleted vocal tour record {}", vocal_tour_id),
        Err(e) => {
            tracing::error!("Failed to delete vocal tour {} from database: {}", vocal_tour_id, e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to delete vocal tour."));
        }
    }

    // 5. Delete associated assets from DB and GCS.
    // We will proceed even if some of these fail, logging errors.
    let gcs_bucket_result = std::env::var("GCS_BUCKET_MICROSITES");

    for asset in assets_to_delete {
        // Delete from DB
        match crate::queries::assets::delete_asset::delete_asset(&pool, asset.id).await {
            Ok(_) => tracing::info!("Successfully deleted asset record {}", asset.id),
            Err(e) => tracing::warn!("Failed to delete asset record {}: {}", asset.id, e),
        }

        // Delete from GCS
        if !asset.gcs_object_name.is_empty() {
            match &gcs_bucket_result {
                Ok(bucket_name) if !bucket_name.is_empty() => {
                    let gcs_client_clone = gcs_client.clone();
                    let bucket_clone = bucket_name.clone();
                    let gcs_object_name = asset.gcs_object_name.clone();
                    let asset_id = asset.id;
                    tokio::spawn(async move {
                        match gcs_client_clone.delete_object(&bucket_clone, &gcs_object_name).await {
                            Ok(_) => tracing::info!("Successfully deleted GCS object {} for asset {}", gcs_object_name, asset_id),
                            Err(e) => tracing::warn!("Failed to delete GCS object {} for asset {}: {}", gcs_object_name, asset_id, e),
                        }
                    });
                }
                _ => {
                    tracing::error!("GCS_BUCKET_MICROSITES not set. Cannot delete GCS object for asset {}", asset.id);
                }
            }
        }
    }

    actix_web::HttpResponse::NoContent().finish()
}