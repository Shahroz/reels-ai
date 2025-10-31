//! Handler for creating a new asset, including GCS upload.
//!
//! Defines the `create_asset` HTTP handler under `/api/assets`.
//! This handler now takes asset content, uploads it to GCS,
//! and then records the asset metadata in the database.
//!
//! ## File Size Considerations
//!
//! **IMPORTANT**: This endpoint accepts files as base64-encoded content in JSON payloads.
//! Base64 encoding inflates file sizes by approximately 33% (4/3 ratio), which significantly
//! impacts the effective file size limits:
//!
//! - JSON limit: 10MB (configured in main.rs)
//! - Effective raw file limit: ~7.5MB (due to base64 expansion + JSON overhead)
//! - Example: 7MB raw file → ~9.3MB base64 → ~9.5MB with JSON = ✅ Accepted
//! - Example: 8MB raw file → ~10.7MB base64 → ~11MB with JSON = ❌ 413 Payload Too Large
//!
//! **TODO**: Consider alternative file upload approaches for better efficiency:
//! - Multipart form uploads (avoid base64 inflation)
//! - Presigned URL uploads (direct client-to-GCS)
//! - Chunked upload support for large files
//! - Stream processing to avoid loading entire files into memory
//!
//! The current base64 approach was chosen for simplicity but has significant drawbacks
//! for large files due to encoding overhead and memory usage.

use actix_web::{post, web, HttpResponse, Responder};
use base64::Engine;
use crate::db::assets::Asset;
use crate::routes::assets::create_asset_request::CreateAssetRequest;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/assets",
    tag = "Assets",
    request_body = CreateAssetRequest, // Assuming this request now includes 'content' field
    responses(
        (status = 201, description = "Asset created and uploaded", body = Asset),
        (status = 400, description = "Bad Request - Invalid input or decoding error"),
        (status = 500, description = "Internal Server Error - GCS Upload or DB Error")
    ),
    security(("user_auth" = []))
)]
#[post("")]
#[instrument(skip(pool, gcs_client, claims, req))]
pub async fn create_asset(
    pool: web::Data<sqlx::PgPool>,
    gcs_client: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<crate::routes::assets::create_asset_request::CreateAssetRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    // Assuming CreateAssetRequest now contains name, r#type, content (base64 encoded), collection_id, and is_public
    let crate::routes::assets::create_asset_request::CreateAssetRequest {
        name,
        r#type,
        content,
        collection_id,
        is_public,
        url: _, // Ignore the url field as we generate it from GCS
    } = req.into_inner();

    // Parse collection_id if provided
    let parsed_collection_id = collection_id.as_ref().and_then(|id| {
        uuid::Uuid::parse_str(id).ok()
    });

    // Handle is_public field with admin check
    let is_public = if let Some(requested_public) = is_public {
        if requested_public && !claims.is_admin {
            log::warn!(
                "Non-admin user {} attempted to create public asset",
                claims.user_id
            );
            return HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse {
                error: "Only administrators can create public assets".into(),
            });
        }
        requested_public
    } else {
        false // Default for new assets
    };

    // For public assets, set user_id to NULL so they're accessible to all users
    let asset_user_id = if is_public { None } else { Some(user_id) };

    // 1. Read GCS bucket name from environment variable
    let bucket_name = match std::env::var("GCS_BUCKET_MICROSITES") {
        Ok(bucket) => bucket,
        Err(e) => {
            log::error!("Failed to get GCS_BUCKET_MICROSITES env var: {e}");
            return HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Server configuration error".into(),
                },
            );
        }
    };

    // 2. Generate a new asset ID
    let asset_id = uuid::Uuid::new_v4();

    // 3. Decode base64 content
    // NOTE: Base64 encoding inflates file sizes by ~33%. A 7MB file becomes ~9.3MB base64.
    // This affects the effective file size limit relative to the JSON payload limit (10MB).
    // Consider multipart uploads for better efficiency with large files.
    let decoded_content = match base64::engine::general_purpose::STANDARD.decode(&content) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!(
                "Failed to decode base64 content for user {user_id}: {e}"
            );
            return HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                error: "Invalid base64 content".into(),
            });
        }
    };

    // 4. Extract metadata from the file content (before uploading to avoid borrowing issues)
    let metadata = match crate::services::metadata_extraction::extract_asset_metadata(
        &decoded_content,
        &r#type,
    ).await {
        Ok(meta) => meta,
        Err(e) => {
            log::warn!("Failed to extract metadata for asset {asset_id}: {e}");
            None // Continue without metadata if extraction fails
        }
    };

    // 5. Determine file extension and construct GCS object name
    // Using "bin" as default if no extension is found. Consider more robust handling if needed.
    let extension = name.split('.').next_back().unwrap_or("bin");
    let gcs_object_name = format!("{user_id}/{asset_id}.{extension}");

    // 6. Upload to GCS
    let gcs_url = match gcs_client
        .upload_raw_bytes(
            &bucket_name,
            &gcs_object_name,
            &r#type, // Use the provided type as content_type
            decoded_content,
            false,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload asset to GCS for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to upload asset".into(),
                },
            );
        }
    };

    // 7. Insert asset metadata into the database
    let result = crate::queries::assets::create_asset::create_asset(
        &pool,
        asset_id,
        asset_user_id,
        &name,
        &r#type,
        &gcs_object_name,
        &gcs_url,
        parsed_collection_id,
        metadata,
        is_public,
    )
    .await;

    match result {
        Ok(asset) => {
            // Update credit reward progress for asset upload
            if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
                &pool,
                user_id,
                crate::app_constants::credits_constants::CreditRewardActionTypes::UPLOAD_ASSETS,
                1,
            ).await {
                log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
            }
            
            HttpResponse::Created().json(asset)
        },
        Err(e) => {
            log::error!("Error creating asset DB record for user {user_id}: {e}");
            // Consider adding logic here to delete the uploaded GCS object if the DB insert fails
            HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to save asset metadata".into(),
            })
        }
    }
}

// Need to ensure CreateAssetRequest struct (presumably in create_asset_request.rs)
// has been updated to include `content: String`.
// Example of what that struct might look like:
/*
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAssetRequest {
    #[schema(example = "My Document.pdf")]
    pub name: String,
    #[schema(example = "application/pdf")]
    pub r#type: String,
    #[schema(example = "JVBERi0xLjQKJeLjz9MKMyAwIG9iago8PCAvVHlwZS...", description = "Base64 encoded file content")]
    pub content: String,
}
*/

// Import necessary crates at the top level of the module or crate if not already present
// use uuid;
// use base64::{engine::general_purpose, Engine as _};
// use std::env;
// use log;
