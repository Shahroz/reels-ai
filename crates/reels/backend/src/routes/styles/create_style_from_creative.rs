//! Defines the handler for creating a new style resource from an existing creative.
//!
//! This function handles POST requests to `/api/styles/from-creative`.
//! It takes a creative ID and a new style name from the request body.
//! It fetches the HTML content of the specified creative, then uses that
//! content to create a new style, saving it to GCS and the database.
//! Returns the created Style object with access level information.

// No 'use' statements, using fully qualified paths as per guidelines.

use crate::routes::styles::create_style_from_creative_request::CreateStyleFromCreativeRequest;
use crate::services::gcs::parse_gcs_url::parse_gcs_url;
use crate::auth::tokens::Claims;
use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;
use crate::routes::error_response::ErrorResponse;
use crate::routes::styles::responses::StyleResponse;
use actix_web::{post, web, HttpResponse, Responder};
use base64::Engine;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use crate::services::gcs::convert_to_pages_url::convert_to_pages_url;

#[utoipa::path(
    post,
    path = "/api/styles/from-creative",
    request_body = CreateStyleFromCreativeRequest,
    responses(
        (status = 201, description = "Style created from creative successfully", body = StyleResponse),
        (status = 400, description = "Bad request (e.g., invalid creative_id, missing name)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Creative not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[post("/from-creative")]
#[instrument(skip(pool, gcs, screenshot_service, claims, req))]
pub async fn create_style_from_creative(
    pool: web::Data<PgPool>,
    gcs: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    screenshot_service: web::Data<std::sync::Arc<dyn crate::services::screenshot::screenshot_service::ScreenshotService>>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateStyleFromCreativeRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request_data = req.into_inner();
    let organization_id = request_data.organization_id;
    let creative_id = request_data.creative_id;
    let new_style_name = request_data.name;

    // 1. Fetch the source creative's html_url
    let creative_record = match sqlx::query!(
        r#"SELECT html_url FROM creatives WHERE id = $1"#,
        creative_id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            log::warn!("Creative not found for ID: {creative_id}");
            return HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Source creative not found: {creative_id}"),
            });
        }
        Err(e) => {
            log::error!("DB error fetching creative {creative_id}: {e:?}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch source creative".to_string(),
            });
        }
    };
    let source_creative_html_url = creative_record.html_url;

    // 2. Fetch HTML content from the creative's html_url
    log::info!(
        "Fetching HTML content for new style '{}' from creative's URL: {}",
        new_style_name,
        source_creative_html_url
    );
    let fetched_html_content = match parse_gcs_url(&source_creative_html_url) {
        Ok((bucket_name, object_name)) => {
            match gcs
                .get_ref()
                .as_ref()
                .download_object_as_string(&bucket_name, &object_name)
                .await
            {
                Ok(html) => html,
                Err(e) => {
                    log::error!(
                        "Failed to download creative HTML from GCS bucket '{bucket_name}', object '{object_name}': {e:?}"
                    );
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to read creative HTML from storage".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            log::error!("Failed to parse creative HTML URL '{source_creative_html_url}': {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Invalid creative HTML URL format".to_string(),
            });
        }
    };

    // 3. Proceed with style creation logic (similar to create_style.rs)
    let bucket = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server configuration error: Missing GCS_BUCKET.".into(),
            });
        }
    };
    let new_style_id = uuid::Uuid::new_v4();

    // Process embedded base64 data URIs
    let processed_html_result = crate::utils::html_minimizer::process_image_data_uris::process_image_data_uris(
        &fetched_html_content,
        gcs.get_ref().as_ref(),
        &bucket,
        new_style_id, // Use the new style_id for GCS paths
    )
    .await;

    let final_html_content = match processed_html_result {
        Ok(processed_html) => processed_html,
        Err(e) => {
            log::error!("Error processing data URIs for style derived from creative {creative_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to process style content: {e}"),
            });
        }
    };

    // Upload HTML for the new style
    let html_object_path = format!("styles/{new_style_id}/style.html");
    let html_bytes = final_html_content.clone().into_bytes();
    let new_style_html_gcs_url = match gcs.upload_raw_bytes(&bucket, &html_object_path, "text/html", html_bytes, true, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload new style HTML to GCS: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to store new style HTML.".into(),
            });
        }
    };

    // Convert to pages.bounti.ai URL for consistent use
    let new_style_html_url = convert_to_pages_url(&new_style_html_gcs_url);

    // Take screenshot of the new style's HTML (using pages.bounti.ai URL)
    let screenshot_base64 = match screenshot_service.screenshot_website(&new_style_html_url, true).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to screenshot new style HTML (from creative {creative_id}): {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to screenshot new style.".into(),
            });
        }
    };
    let screenshot_data = match base64::engine::general_purpose::STANDARD.decode(&screenshot_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Invalid base64 in new style screenshot data: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process new style screenshot data.".into(),
            });
        }
    };

    // Upload screenshot for the new style
    let screenshot_object_path = format!("styles/{new_style_id}/screenshot.png");
    let new_style_screenshot_gcs_url = match gcs.upload_raw_bytes(&bucket, &screenshot_object_path, "image/png", screenshot_data, false, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload new style screenshot to GCS: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to store new style screenshot.".into(),
            });
        }
    };

    // Convert screenshot URL to pages.bounti.ai format
    let new_style_screenshot_url = convert_to_pages_url(&new_style_screenshot_gcs_url);

    #[derive(sqlx::FromRow, Debug)]
    struct CreatedStyleDetails {
        id: Uuid,
        user_id: Option<Uuid>,  // Can be NULL for public styles
        name: String,
        html_url: String,
        screenshot_url: String,
        is_public: bool,        // NOT NULL in database
        created_at: DateTime<Utc>,  // NOT NULL in database
        updated_at: DateTime<Utc>,  // NOT NULL in database
        creator_email: Option<String>,  // From LEFT JOIN users
        current_user_access_level: Option<String>,  // CASE statement can return NULL
    }

    // Insert new style record into the database
    let insert_result = sqlx::query_as!(
        CreatedStyleDetails,
        r#"
        WITH inserted_style AS (
            INSERT INTO styles (id, user_id, name, html_url, screenshot_url, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
        )
        SELECT
            i_s.id as "id!", i_s.user_id, i_s.name as "name!", i_s.html_url as "html_url!", 
            i_s.screenshot_url as "screenshot_url!", i_s.is_public as "is_public!",
            i_s.created_at as "created_at!", i_s.updated_at as "updated_at!", 
            u.email as "creator_email?",
            'owner'::text AS "current_user_access_level?"
        FROM inserted_style i_s
        LEFT JOIN users u ON i_s.user_id = u.id
        "#,
        new_style_id,
        Some(user_id),
        new_style_name.clone(),
        new_style_html_url.clone(),
        new_style_screenshot_url.clone(),
        false // is_public = false for private styles
    )
    .fetch_one(&**pool)
    .await;

    match insert_result {
        Ok(details) => {
            log::info!(
                "Successfully created style '{}' (ID: {}) from creative ID: {}",
                new_style_name,
                details.id,
                creative_id
            );

            // Consume credits before returning response
            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_STYLE;
            let deduction_params = CreditChangesParams {
                user_id,
                organization_id,
                credits_to_change: BigDecimal::from(credits_to_consume),
                action_source: "api".to_string(),
                action_type: "generate_style_from_creative".to_string(),
                entity_id: Some(details.id),
            };
            if let Err(e) = deduct_user_credits_with_transaction(&pool, deduction_params).await {
                log::error!("Failed to deduct {} credits for user {} after creating style from creative: {}", credits_to_consume, user_id, e);
            }

            let response = StyleResponse {
                style: crate::db::styles::Style {
                    id: details.id,
                    user_id: details.user_id,
                    name: details.name,
                    html_url: details.html_url,
                    screenshot_url: details.screenshot_url,
                    is_public: details.is_public,
                    created_at: details.created_at,
                    updated_at: details.updated_at,
                },
                creator_email: details.creator_email,
                current_user_access_level: details.current_user_access_level,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!(
                "Error creating style (from creative {creative_id}) for user {user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create style in database".into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    // Unit tests for handlers are typically more complex and might involve mocking.
    // For now, ensure the module structure is present as per guidelines.
    // Example:
    // #[actix_web::rt::test]
    // async fn test_example_case() {
    //     // Mock services, setup DB state, call handler, assert response
    // }
}