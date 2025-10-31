//! Handler for confirming asset uploads after direct GCS upload.
//!
//! Defines the `confirm_upload` HTTP handler under `/api/assets/confirm-upload`.
//! This handler verifies that an asset has been successfully uploaded to GCS
//! and creates the corresponding database record.

use actix_web::{post, web, HttpResponse, Responder};
use crate::routes::assets::confirm_upload_request::ConfirmUploadRequest;
use crate::routes::assets::confirm_upload_response::ConfirmUploadResponse;
use crate::services::gcs::gcs_client::GCSClient;
use crate::services::gcs::gcs_operations::GCSOperations;
use crate::services::photo_extraction::convert_heic_on_gcs::convert_heic_on_gcs;
use crate::services::photo_extraction::convert_dng_on_gcs::convert_dng_on_gcs;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/assets/confirm-upload",
    tag = "Assets",
    request_body = ConfirmUploadRequest,
    responses(
        (status = 200, description = "Asset upload confirmed and registered", body = ConfirmUploadResponse),
        (status = 404, description = "Asset not found in GCS - upload may have failed"),
        (status = 500, description = "Internal Server Error - Failed to register asset")
    ),
    security(("user_auth" = []))
)]
#[post("/confirm-upload")]
#[instrument(skip(pool, gcs_client, claims, req, http_req, session_manager))]
pub async fn confirm_upload(
    pool: web::Data<sqlx::PgPool>,
    gcs_client: web::Data<std::sync::Arc<dyn GCSOperations>>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<ConfirmUploadRequest>,
    http_req: actix_web::HttpRequest,
    session_manager: web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> impl Responder {
    let processing_start = std::time::Instant::now();
    let user_id = claims.user_id;
    let ConfirmUploadRequest {
        asset_id,
        file_name,
        content_type,
        is_public,
    } = req.into_inner();
    
    // Extract request context for event tracking  
    #[cfg(feature = "events")]
    let request_context = extract_request_context_for_assets(&http_req, user_id, &session_manager).await;

    // Handle is_public field with admin check
    let is_public = if let Some(requested_public) = is_public {
        if requested_public && !claims.is_admin {
            log::warn!(
                "Non-admin user {} attempted to create public asset via confirm upload",
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

    // 1. Validate the upload request for security (derive secure extension)
    let validation_result = crate::routes::assets::upload_validation::validate_upload_request(
        &file_name,
        0, // Size is not relevant for confirmation, already validated at upload URL generation
        &content_type,
    );

    if !validation_result.is_valid {
        let error_message = validation_result.error_message.unwrap_or_else(|| "Invalid upload request".to_string());
        log::warn!(
            "Upload confirmation validation failed for user {user_id} - Asset: {asset_id}, File: {file_name} ({content_type}), Error: {error_message}"
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

    // 3. Get the concrete GCS client to use the object_exists method
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

    // 4. Use secure extension from validation (not client-provided extension)
    let secure_extension = &validation_result.secure_extension;
    let object_name = format!("{user_id}/{asset_id}.{secure_extension}");

    // 5. Verify the object exists in GCS
    let object_exists = match gcs_concrete_client.object_exists(&bucket_name, &object_name).await {
        Ok(exists) => exists,
        Err(e) => {
            log::error!("Failed to verify object existence for asset {asset_id}: {e}");
            return HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to verify upload".into(),
                },
            );
        }
    };

    if !object_exists {
        log::warn!("Asset {asset_id} not found in GCS at path: {object_name}");
        return HttpResponse::NotFound().json(
            crate::routes::error_response::ErrorResponse {
                error: "Asset not found in storage. Upload may have failed.".into(),
            },
        );
    }

    // 6. Handle RAW image conversion if needed (HEIC, DNG)
    let (final_object_name, final_content_type, final_file_name) = if content_type == "image/heic" {
        log::info!("Detected HEIC file for asset {asset_id}, starting conversion to web-compatible format");
        
        match convert_heic_on_gcs(&gcs_concrete_client, &bucket_name, &object_name, None).await {
            Ok(conversion_result) => {
                log::info!(
                    "Successfully converted HEIC for asset {asset_id}: {} -> {}",
                    object_name,
                    conversion_result.new_object_name
                );
                
                // Update file name to have the correct extension based on conversion result
                let new_file_name = file_name
                    .replace(".heic", &format!(".{}", conversion_result.new_extension))
                    .replace(".HEIC", &format!(".{}", conversion_result.new_extension));
                
                (
                    conversion_result.new_object_name,
                    conversion_result.new_content_type,
                    new_file_name
                )
            }
            Err(e) => {
                log::error!("Failed to convert HEIC image for asset {asset_id}: {e:?}");
                return HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: "Failed to convert HEIC image to web-compatible format".into(),
                    },
                );
            }
        }
    } else if content_type == "image/x-adobe-dng" {
        log::info!("Detected DNG file for asset {asset_id}, starting conversion to web-compatible format");
        
        match convert_dng_on_gcs(&gcs_concrete_client, &bucket_name, &object_name, None).await {
            Ok(conversion_result) => {
                log::info!(
                    "Successfully converted DNG for asset {asset_id}: {} -> {}",
                    object_name,
                    conversion_result.new_object_name
                );
                
                // Update file name to have the correct extension based on conversion result
                let new_file_name = file_name
                    .replace(".dng", &format!(".{}", conversion_result.new_extension))
                    .replace(".DNG", &format!(".{}", conversion_result.new_extension));
                
                (
                    conversion_result.new_object_name,
                    conversion_result.new_content_type,
                    new_file_name
                )
            }
            Err(e) => {
                log::error!("Failed to convert DNG image for asset {asset_id}: {e:?}");
                return HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: "Failed to convert DNG image to web-compatible format".into(),
                    },
                );
            }
        }
    } else {
        // No conversion needed, use original values
        (object_name, content_type.clone(), file_name.clone())
    };

    // 7. Generate the public URL for the asset
    // TODO: Consider implementing convert_to_pages_url() for consistency with creatives
    // to support pages.bounti.ai URLs when assets move to bounti_prod_narrativ_public bucket
    let public_url = format!("https://storage.googleapis.com/{bucket_name}/{final_object_name}");

    // 8. Create the asset record in the database
    // Note: This function confirms an upload that already happened, so we don't have
    // access to the file content for metadata extraction
    let asset_result = crate::queries::assets::create_asset::create_asset(
        &pool,
        asset_id,
        asset_user_id,
        &final_file_name,
        &final_content_type,
        &final_object_name,
        &public_url,
        None, // collection_id - not supported in confirm_upload flow yet
        None, // metadata - not available in confirm upload flow
        is_public,
    )
    .await;

    match asset_result {
        Ok(asset) => {
            log::info!(
                "Asset {asset_id} successfully confirmed and registered for user {user_id} - File: {final_file_name} ({final_content_type})"
            );

            // Update credit reward progress for asset upload
            if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
                &pool,
                user_id,
                crate::app_constants::credits_constants::CreditRewardActionTypes::UPLOAD_ASSETS,
                1,
            ).await {
                log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
            }

            // Calculate upload duration (approximate - this could be improved with actual upload timing)
            let upload_duration_seconds = processing_start.elapsed().as_secs_f64();
            
            // Determine if conversion was applied
            let conversion_applied = if final_content_type != content_type {
                Some(format!("{}_to_{}", 
                    content_type.split('/').nth(1).unwrap_or("unknown"),
                    final_content_type.split('/').nth(1).unwrap_or("unknown")
                ))
            } else {
                None
            };
            
            // Log asset upload completed event
            #[cfg(feature = "events")]
            {
                let _ = crate::services::events_service::asset_events::log_asset_upload_completed(
                    &pool,
                    user_id,
                    asset_id,
                    &asset,
                    upload_duration_seconds,
                    conversion_applied.as_deref(),
                    &request_context,
                    processing_start,
                ).await;
            }

            HttpResponse::Ok().json(ConfirmUploadResponse {
                asset,
                status: "confirmed".to_string(),
                message: "Asset successfully registered".to_string(),
            })
        }
        Err(e) => {
            log::error!("Failed to create asset record for asset {asset_id}: {e}");
            HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to register asset".into(),
                },
            )
        }
    }
}

/// Extract request context for asset event tracking
#[cfg(feature = "events")]
async fn extract_request_context_for_assets(
    http_req: &actix_web::HttpRequest,
    user_id: uuid::Uuid,
    session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> crate::services::events_service::request_context::RequestData {
    // Extract basic request info
    let method = http_req.method().to_string();
    let path = http_req.path().to_string();
    let query_string = http_req.query_string().to_string();
    let scheme = if http_req.connection_info().scheme() == "https" { "https" } else { "http" };
    let host = http_req.connection_info().host().to_string();
    let full_url = format!("{}://{}{}", scheme, host, path);
    
    // Extract headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in http_req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    // Extract IP address
    let connection_info = http_req.connection_info();
    let ip_address = connection_info.realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .map(|addr| addr.split(':').next().unwrap_or(addr).to_string());
    
    // Extract user agent
    let user_agent = headers.get("user-agent").cloned();
    
    // Get session ID using session manager
    let session_id = match session_manager.get_or_create_session(user_id).await {
        Ok(session) => Some(session),
        Err(e) => {
            log::warn!("Failed to get session for user {}: {}", user_id, e);
            None
        }
    };
    
    crate::services::events_service::request_context::RequestData {
        method,
        path,
        full_url,
        query_string,
        headers,
        query_params: serde_json::json!({}),
        user_agent,
        ip_address,
        real_ip: None,
        forwarded_for: None,
        scheme: scheme.to_string(),
        host,
        port: None,
        http_version: format!("{:?}", http_req.version()),
        content_type: None,
        content_length: None,
        content_encoding: None,
        accept_language: None,
        accept_encoding: None,
        request_body: None,
        request_body_size: None,
        request_body_truncated: false,
        user_registration_date: None,
        cookies: std::collections::HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        user_id: Some(user_id),
        session_id,
    }
} 