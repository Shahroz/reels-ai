//! Handler for generating signed upload URLs for asset uploads.
//!
//! Defines the `get_upload_url` HTTP handler under `/api/assets/upload-url`.
//! This handler generates a signed URL that allows clients to upload
//! assets directly to Google Cloud Storage, bypassing the application server.

use actix_web::{post, web, HttpResponse, Responder};
use crate::routes::assets::get_upload_url_request::GetUploadUrlRequest;
use crate::routes::assets::get_upload_url_response::GetUploadUrlResponse;
use crate::services::gcs::gcs_client::GCSClient;
use crate::services::gcs::gcs_operations::GCSOperations;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/assets/upload-url",
    tag = "Assets",
    request_body = GetUploadUrlRequest,
    responses(
        (status = 200, description = "Signed upload URL generated successfully", body = GetUploadUrlResponse),
        (status = 400, description = "Bad Request - Invalid input parameters"),
        (status = 500, description = "Internal Server Error - Failed to generate signed URL")
    ),
    security(("user_auth" = []))
)]
#[post("/upload-url")]
#[instrument(skip(gcs_client, claims, req))]
pub async fn get_upload_url(
    gcs_client: web::Data<std::sync::Arc<dyn GCSOperations>>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<GetUploadUrlRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let GetUploadUrlRequest {
        file_name,
        file_size,
        content_type,
    } = req.into_inner();

    // 1. Validate the upload request
    let validation_result = crate::routes::assets::upload_validation::validate_upload_request(
        &file_name,
        file_size,
        &content_type,
    );

    if !validation_result.is_valid {
        let error_message = validation_result.error_message.unwrap_or_else(|| "Invalid upload request".to_string());
        log::warn!(
            "Upload validation failed for user {user_id} - File: {file_name} ({content_type}), Size: {file_size}, Error: {error_message}"
        );
        return HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                error: error_message,
            },
        );
    }

    // 2. Read GCS bucket name from environment variable
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

    // 3. Generate a new asset ID
    let asset_id = uuid::Uuid::new_v4();

    // 4. Use normalized extension from validation
    let file_extension = &validation_result.secure_extension;

    // 5. Get the concrete GCS client to use the generate_signed_upload_url method
    let gcs_concrete_client = match gcs_client.as_any().downcast_ref::<GCSClient>() {
        Some(client) => client.clone(),
        None => {
            log::error!("Failed to downcast GCS client to concrete type");
            return HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Internal service error".into(),
                },
            );
        }
    };

    // 6. Generate object name using the same pattern as confirm_upload
    let object_name = format!("{user_id}/{asset_id}.{file_extension}");

    // 7. Generate the signed URL with 15 minute expiration
    let expires_in = std::time::Duration::from_secs(15 * 60);
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(15 * 60);

    let signed_url = match gcs_concrete_client.generate_signed_upload_url(
        &bucket_name,
        &object_name,
        Some(content_type.clone()),
        expires_in,
    ).await {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to generate signed URL for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to generate upload URL".into(),
                },
            );
        }
    };

    // 8. Log the upload URL generation with asset category for monitoring
    log::info!(
        "Generated signed upload URL for user {} - Asset ID: {}, File: {} ({}), Size: {} bytes, Category: {:?}, Size limit: {} bytes",
        user_id,
        asset_id,
        file_name,
        content_type,
        file_size,
        validation_result.asset_category,
        validation_result.size_limit
    );

    // 9. Return the signed URL response
    HttpResponse::Ok().json(GetUploadUrlResponse {
        asset_id,
        upload_url: signed_url,
        upload_method: "PUT".to_string(),
        expires_at,
        object_name,
    })
} 