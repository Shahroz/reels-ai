//! Defines the handler for creating a new style resource.
//!
//! This function handles POST requests to `/api/styles`. It takes style details
//! from the request body. If `html_content` is provided, it's used directly.
//! If `source_url` is provided (and `html_content` is not), it fetches the HTML
//! content from the URL using Zyte before saving. Inserts a new record
//! into the database for the authenticated user. Returns the created Style object
//! with access level information.
//! Requires DB pool, user claims, and JSON payload extractor.

use crate::auth::tokens::Claims;
use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;
use crate::routes::error_response::ErrorResponse;
use crate::routes::styles::create_style_request::CreateStyleRequest;
use crate::routes::styles::responses::StyleResponse;
use crate::routes::styles::validation::validate_style_name::validate_style_name;
use crate::zyte::zyte::ZyteClient;
use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use sqlx::PgPool;
use std::env; // Added for environment variables
// use std::collections::HashMap; // For tracking replacements - UNUSED
// use scraper::{Html, Selector}; // For HTML parsing - UNUSED
// use mime_guess; // For deriving file extensions - UNUSED
// use crate::utils::gcs_utils::upload_image_from_bytes_async; // Commented out due to missing gcs_utils.rs
use tracing::instrument;
use crate::services::gcs::convert_to_pages_url::convert_to_pages_url;

#[utoipa::path(
    post,
    path = "/api/styles",
    request_body = CreateStyleRequest,
    params(
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to deduct credits from organization instead of user")
    ),
    responses(
        (status = 201, description = "Style created", body = StyleResponse),
        (status = 400, description = "Bad request (e.g., missing content/URL, fetch failed)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error (DB error, Zyte error)", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[post("")]
#[instrument(skip(pool, gcs, screenshot_service, claims, req))]
pub async fn create_style(
    pool: web::Data<PgPool>,
    gcs: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    screenshot_service: web::Data<std::sync::Arc<dyn crate::services::screenshot::screenshot_service::ScreenshotService>>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateStyleRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request_data = req.into_inner();
    let organization_id = request_data.organization_id;
    let style_name = request_data.name.clone(); // Clone name for logging if needed later

    // Validate style name
    if let Err(validation_error) = validate_style_name(&style_name) {
        return validation_error;
    }

    // Determine if this is a public style and set user_id accordingly
    let is_public = request_data.is_public.unwrap_or(false);
    let style_user_id = if is_public {
        None // Public styles have user_id = NULL so they're accessible to all users
    } else {
        Some(user_id) // Private styles belong to the creating user
    };

    // Determine HTML content: Use provided HTML, fetch from URL, or error out.
    let html_result = match (request_data.html_content, request_data.source_url) {
        // Reject if both html_content and source_url are provided
        (Some(_), Some(_)) => {
            log::warn!("Create style request failed for user {user_id}: Both html_content and source_url provided");
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Only one of html_content or source_url can be provided, not both.".into(),
            });
        }
        // Use source_url if only it is provided
        (None, Some(url)) if !url.trim().is_empty() => {
            // Validate URL format before proceeding
            if !url.starts_with("http://") && !url.starts_with("https://") {
                log::warn!("Create style request failed for user {user_id}: Invalid URL format: {url}");
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Invalid URL format. URLs must start with http:// or https://.".into(),
                });
            }
            
            log::info!(
                "Fetching HTML content for style '{style_name}' from URL: {url}"
            );
            // Fetch HTML using Zyte client, similar to clone_style logic
            let api_key = match env::var("ZYTE_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    log::error!("ZYTE_API_KEY environment variable not set.");
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Server configuration error: Missing API key.".into(),
                    });
                }
            };
            let zyte_client = ZyteClient::new(api_key);
            // Using extract_inline_styles_v2 as a likely candidate based on clone_style // Removed unused import comment
            match zyte_client.extract_styles_with_fallback(&url).await {
                Ok(fetched_html) => {
                    log::info!("Successfully fetched HTML from {url}");
                    Ok(fetched_html)
                }
                Err(e) => {
                    log::error!("Failed to fetch HTML from URL {url}: {e}");
                    Err(HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to fetch style from source URL: {e}"),
                    }))
                }
            }
        }
        // Use html_content if only it is provided
        (Some(html), None) => {
            if html.trim().is_empty() {
                log::warn!("Create style request failed for user {user_id}: Empty html_content provided");
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "html_content cannot be empty.".into(),
                });
            }
            log::info!("Using provided html_content for style '{style_name}'");
            Ok(html)
        }
        // Handle missing content cases
        (None, None) | (None, Some(_)) => { // Covers (None, None) and (None, Some(empty_url))
            log::warn!(
                "Create style request failed for user {user_id}: Missing html_content and source_url"
            );
            Err(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Either html_content or source_url must be provided.".into(),
            }))
        }
    };

    let mut final_html_content = match html_result { // Make mutable
        Ok(html) => html,
        Err(response) => return response, // Return the error response directly
    };
    // Upload style HTML and screenshot to GCS
    let bucket = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server configuration error: Missing GCS_BUCKET.".into(),
            });
        }
    };
    let style_id = Uuid::new_v4();
    
    // --- Process embedded base64 data URIs using the utility function ---
    let processed_html_result = crate::utils::minimize_large_html_content::minimize_large_html_content(
        &final_html_content,
        gcs.get_ref().as_ref(),
        &bucket,
        style_id,
        300_000,
        1000
    ).await;

    final_html_content = match processed_html_result {
        Ok(processed_html) => processed_html,
        Err(e) => {
            log::error!("Error processing data URIs for style {style_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to process style content: {e}"),
            });
        }
    };

    // Upload HTML
    let html_object = format!("styles/{style_id}/style.html");
    let html_bytes = final_html_content.clone().into_bytes();
    log::info!("Attempting to upload style HTML to GCS. Bucket: '{}', Object: '{}', Size: {} bytes", bucket, html_object, html_bytes.len());
    let html_gcs_url = match gcs.upload_raw_bytes(&bucket, &html_object, "text/html", html_bytes, true, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => {
            log::info!("Successfully uploaded style HTML to GCS: {url}");
            url
        },
        Err(e) => {
            log::error!("Failed to upload style HTML to GCS. Bucket: '{bucket}', Object: '{html_object}', Error: {e:#}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to store style HTML.".into(),
            });
        }
    };

    // Convert to pages.bounti.ai URL for consistent use
    let html_url = convert_to_pages_url(&html_gcs_url);

    // Take screenshot of the uploaded HTML (using pages.bounti.ai URL)
    let screenshot_base64 = match screenshot_service.screenshot_website(&html_url, true).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to screenshot style HTML: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to screenshot style.".into(),
            });
        }
    };
    let screenshot_data = match general_purpose::STANDARD.decode(&screenshot_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Invalid base64 in screenshot data: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process screenshot data.".into(),
            });
        }
    };
    // Upload screenshot
    let screenshot_object = format!("styles/{style_id}/screenshot.png");
    log::info!("Attempting to upload screenshot to GCS. Bucket: '{}', Object: '{}', Size: {} bytes", bucket, screenshot_object, screenshot_data.len());
    let screenshot_gcs_url = match gcs.upload_raw_bytes(&bucket, &screenshot_object, "image/png", screenshot_data, false, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic).await {
        Ok(url) => {
            log::info!("Successfully uploaded screenshot to GCS: {url}");
            url
        },
        Err(e) => {
            log::error!("Failed to upload screenshot to GCS. Bucket: '{bucket}', Object: '{screenshot_object}', Error: {e:#}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to store style screenshot.".into(),
            });
        }
    };

    // Convert screenshot URL to pages.bounti.ai format
    let screenshot_url = convert_to_pages_url(&screenshot_gcs_url);

    #[derive(sqlx::FromRow, Debug)]
    struct CreatedStyleDetails {
        id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        html_url: String,
        screenshot_url: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
        is_public: Option<bool>,
    }

    // Proceed with database insertion
    let result = sqlx::query_as!(
        CreatedStyleDetails,
        r#"
        WITH inserted_style AS (
            INSERT INTO styles (id, user_id, name, html_url, screenshot_url, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
        )
        SELECT 
            i_s.id as "id!", i_s.user_id, i_s.name as "name!", i_s.html_url as "html_url!", i_s.screenshot_url as "screenshot_url!", 
            i_s.created_at as "created_at!", i_s.updated_at as "updated_at!", u.email as "creator_email?",
            CASE
                WHEN i_s.user_id = $7 THEN 'owner'::text
                ELSE NULL::text
            END AS "current_user_access_level?",
            i_s.is_public as "is_public?"
        FROM inserted_style i_s
        LEFT JOIN users u ON i_s.user_id = u.id
        "#,
        style_id,
        style_user_id,
        style_name,
        html_url.clone(),
        screenshot_url.clone(),
        is_public,
        user_id
    )
    .fetch_one(&**pool)
    .await;

    match result {
        Ok(details) => {
            // Consume credits before returning response
            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_STYLE;
            let deduction_params = CreditChangesParams {
                user_id,
                organization_id,
                credits_to_change: BigDecimal::from(credits_to_consume),
                action_source: "api".to_string(),
                action_type: "generate_style".to_string(),
                entity_id: Some(details.id.clone()),
            };
            if let Err(e) = deduct_user_credits_with_transaction(&pool, deduction_params).await {
                log::error!("Failed to deduct {} credits for user {} after creating style: {}", credits_to_consume, user_id, e);
            }

            let response = StyleResponse {
                style: crate::db::styles::Style {
                    id: details.id,
                    user_id: details.user_id,
                    name: details.name,
                    html_url: details.html_url,
                    screenshot_url: details.screenshot_url,
                    created_at: details.created_at,
                    updated_at: details.updated_at,
                    is_public: details.is_public.unwrap_or(false),
                },
                creator_email: details.creator_email,
                current_user_access_level: details.current_user_access_level,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!("Error creating style for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create style in database".into(),
            })
        }
    }
}
