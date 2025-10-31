//! Handler for deleting an asset.
//!
//! Defines the `delete_asset` HTTP handler under `/api/assets/{id}`.
//! Includes logic to delete the corresponding object from GCS if configured.

// Note: 'use' statements are forbidden by guidelines. Fully qualified paths are used.
// The #[delete("")] macro from actix_web is used directly.
use tracing::instrument;

#[utoipa::path(
    delete,
    path = "/api/assets/{id}",
    tag = "Assets",
    params(
        ("id" = String, Path, description = "Asset ID")
    ),
    responses(
        (status = 204, description = "No Content. Asset deleted from DB. GCS deletion attempted if applicable."),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Asset not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::delete("{id}")] // Use fully qualified path if needed, but macros often work
#[instrument(skip(pool, gcs_client, path, claims))]
pub async fn delete_asset(
    pool: actix_web::web::Data<sqlx::PgPool>,
    gcs_client: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let authenticated_user_id = claims.user_id;
    let asset_id = path.into_inner();
    log::info!("claims: {claims:?}");

    // --- 1. Fetch asset details for permission check and GCS object name ---
    let asset_to_delete = match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, asset_id).await {
        Ok(Some(asset)) => asset,
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse::from("Asset not found."));
        }
        Err(e) => {
            log::error!("Failed to fetch asset {asset_id} for delete permission check: {e}");
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to retrieve asset for deletion."));
        }
    };

    // --- 2. Permission Check ---
    let can_delete = if asset_to_delete.user_id == Some(authenticated_user_id) {
        // User owns the asset
        true
    } else if asset_to_delete.user_id.is_none() && asset_to_delete.is_public {
        // Public assets can only be deleted by admins
        claims.is_admin
    } else {
        // Check if user has editor access through organization shares
        let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(&pool, authenticated_user_id).await {
            Ok(memberships) => memberships,
            Err(e) => {
                log::error!("Error fetching org memberships (delete_asset) for user {authenticated_user_id}: {e}");
                return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to check permissions."));
            }
        };
        let org_ids: Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
        let org_ids_slice: &[uuid::Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

        match sqlx::query_scalar!(
            r#"
                SELECT EXISTS (
                    SELECT 1 FROM object_shares
                    WHERE object_id = $1 AND object_type = $2 AND access_level = $3
                    AND (
                        (entity_type = 'user' AND entity_id = $4) OR
                        (entity_type = 'organization' AND entity_id = ANY($5))
                    )
                )
            "#,
            asset_id, 
            "asset", 
            crate::db::shares::AccessLevel::Editor as crate::db::shares::AccessLevel,
            authenticated_user_id,
            org_ids_slice
        )
        .fetch_one(pool.as_ref())
        .await {
            Ok(Some(true)) => true,
            Ok(Some(false)) | Ok(None) => false,
            Err(e) => {
                log::error!("Error checking 'editor' share for asset deletion {asset_id}: {e}");
                return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse::from("Failed to verify permissions."));
            }
        }
    };

    if !can_delete {
        log::warn!(
            "User {} attempted to delete asset {} owned by user {:?} (permission denied).",
            authenticated_user_id, asset_id, asset_to_delete.user_id
        );
        return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse::from("Asset not found or you do not have permission to delete this asset."));
    }
    // --- End Permission Check ---

    let gcs_object_name_to_delete = asset_to_delete.gcs_object_name.clone(); // Clone to use after asset_to_delete might be less available

    // --- 3. Execute Database Deletion (now using only asset_id) ---
    let delete_result = crate::queries::assets::delete_asset::delete_asset(&pool, asset_id).await;

    match delete_result {
        Ok(res) if res.rows_affected() > 0 => {
            // --- 4. Conditional GCS Deletion ---
            if !gcs_object_name_to_delete.is_empty() { // Check if there's actually a GCS object name
                match std::env::var("GCS_BUCKET_MICROSITES") {
                    Ok(bucket_name) => {
                        if !bucket_name.is_empty() {
                            match gcs_client
                                .delete_object(&bucket_name, &gcs_object_name_to_delete)
                                .await
                            {
                                Ok(_) => {
                                    log::info!("Successfully deleted GCS object {gcs_object_name_to_delete} in bucket {bucket_name} for asset {asset_id}");
                                }
                                Err(gcs_err) => {
                                    log::warn!(
                                        "Failed to delete GCS object {gcs_object_name_to_delete} in bucket {bucket_name} for asset {asset_id}: {gcs_err}. DB record was deleted."
                                    );
                                }
                            }
                        } else {
                            log::warn!("Skipping GCS deletion for asset {asset_id}: Bucket name is empty.");
                        }
                    }
                    Err(e) => {
                        log::error!("GCS_BUCKET_MICROSITES environment variable not set or invalid: {e}. Cannot delete GCS object for asset {asset_id}. DB record was deleted.");
                    }
                }
            } else {
                log::info!(
                    "No GCS object name associated with asset {asset_id}. Skipping GCS deletion."
                );
            }
            actix_web::HttpResponse::NoContent().finish()
        }
        Ok(_) => {
            log::warn!(
                "Asset {asset_id} found during permission check but not during delete operation."
            );
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: "Asset not found during delete operation".into(),
            })
        }
        Err(e) => {
            log::error!("Error deleting asset {asset_id} from database: {e}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to delete asset from database".into(),
                },
            )
        }
    }
}

// --- Add Tests (if required by guidelines/instruction, but not explicitly asked here) ---
// #[cfg(FALSE)]
// mod tests {
//     // ... tests would go here, using fully qualified paths ...
// }
