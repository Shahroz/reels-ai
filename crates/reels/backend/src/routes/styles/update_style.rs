//! Defines the handler for updating an existing style resource.
//!
//! This function handles PUT requests to `/api/styles/{id}`. It updates the
//! style identified by `id`. If `source_url` is provided, it fetches the HTML
//! from the URL. Ensures the user is the owner or has an 'editor' share.
//! Returns the updated Style object with access level information.
use crate::auth::tokens::Claims;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::error_response::ErrorResponse;
use crate::routes::styles::responses::StyleResponse;
use crate::routes::styles::update_style_request::UpdateStyleRequest;
use crate::routes::styles::validation::validate_style_name::validate_style_name;
use crate::zyte::zyte::ZyteClient;
use actix_web::{put, web, HttpResponse, Responder};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::env;
use tracing::instrument;
use uuid::Uuid;
use crate::services::gcs::convert_to_pages_url::convert_to_pages_url;

#[utoipa::path(
    put,
    path = "/api/styles/{id}",
    params(("id" = String, Path, description = "Style ID")),
    request_body = UpdateStyleRequest,
    responses(
        (status = 200, description = "Style updated", body = StyleResponse),
        (status = 400, description = "Bad request (e.g., fetch failed)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Style not found or access denied", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[put("/{id}")]
#[instrument(skip(pool, gcs, screenshot_service, claims, req))]
pub async fn update_style(
    pool: web::Data<PgPool>,
    gcs: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    screenshot_service: web::Data<std::sync::Arc<dyn crate::services::screenshot::screenshot_service::ScreenshotService>>,
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
    req: web::Json<UpdateStyleRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let style_id = path.into_inner();
   let UpdateStyleRequest {
       name,
       html_content,
       source_url,
       is_public
   } = req.into_inner();

   // Determine user_id based on whether style should be public
   let is_public = is_public.unwrap_or(false);
   let style_user_id = if is_public {
       None // Public styles have user_id = NULL so they're accessible to all users
   } else {
       Some(authenticated_user_id) // Private styles belong to the authenticated user
   };

   // Validate style name
   if let Err(validation_error) = validate_style_name(&name) {
       return validation_error;
   }

   // --- I/O Operations First ---
    // Perform all external operations (Zyte, GCS) before starting the DB transaction.

    let final_html_content = match source_url {
        Some(url) => {
            // Validate URL format before proceeding
            if !url.starts_with("http://") && !url.starts_with("https://") {
                log::warn!("Update style request failed: Invalid URL format: {url}");
                return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid URL format. URLs must start with http:// or https://."));
            }
            
            log::info!(
                "Fetching HTML content for style update '{name}' from URL: {url}"
            );
            let api_key = match env::var("ZYTE_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    log::error!("ZYTE_API_KEY environment variable not set.");
                    return HttpResponse::InternalServerError()
                        .json(ErrorResponse::from("Server configuration error."));
                }
            };
            match ZyteClient::new(api_key).extract_styles_with_fallback(&url).await {
                Ok(fetched_html) => fetched_html,
                Err(e) => {
                    log::error!("Failed to fetch HTML from URL {url} for update: {e}");
                    return HttpResponse::BadRequest().json(ErrorResponse::from(format!(
                        "Failed to fetch style from source URL: {e}"
                    )));
                }
            }
        }
        None => {
            // Validate that html_content is provided and not empty when no source_url
            if html_content.trim().is_empty() {
                log::warn!("Update style request failed: Empty html_content and no source_url provided");
                return HttpResponse::BadRequest().json(ErrorResponse::from("Either source_url or non-empty html_content must be provided"));
            }
            html_content
        }
    };

    let bucket = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Server configuration error."));
        }
    };

    let html_object = format!("styles/{style_id}/style.html");
    let html_gcs_url = match gcs
        .upload_raw_bytes(
            &bucket,
            &html_object,
            "text/html",
            final_html_content.into_bytes(),
            true,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic,
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload style HTML to GCS: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to store style HTML."));
        }
    };

    // Convert to pages.bounti.ai URL for consistent use
    let html_url = convert_to_pages_url(&html_gcs_url);

    let screenshot_base64 = match screenshot_service.screenshot_website(&html_url, true).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to screenshot style HTML: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to screenshot style."));
        }
    };

    let screenshot_data = match STANDARD.decode(&screenshot_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Invalid base64 in screenshot data: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to process screenshot data."));
        }
    };

    let screenshot_object = format!("styles/{style_id}/screenshot.png");
    let screenshot_gcs_url = match gcs
        .upload_raw_bytes(
            &bucket,
            &screenshot_object,
            "image/png",
            screenshot_data,
            false,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic,
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!("Failed to upload screenshot to GCS: {e}");
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to store style screenshot."));
        }
    };

    // Convert screenshot URL to pages.bounti.ai format
    let screenshot_url = convert_to_pages_url(&screenshot_gcs_url);

    // --- Database Transaction ---
    // Now that I/O is done, perform the atomic update and fetch within a transaction.

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction for style update: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Database error."));
        }
    };

    // Get org IDs needed for permission check
    let org_memberships =
        match find_active_memberships_for_user(&pool, authenticated_user_id).await {
            Ok(memberships) => memberships,
            Err(e) => {
                log::error!(
                    "Error fetching org memberships for user {authenticated_user_id}: {e}"
                );
                if let Err(rb_err) = tx.rollback().await {
                    log::error!("Failed to rollback transaction: {rb_err}");
                }
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::from("Failed to check permissions."));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice = if org_ids.is_empty() { &[] } else { &org_ids[..] };

    // Check if style exists and if user can see it (for private styles, only owner can see them)
    let style_visibility = sqlx::query!(
        r#"
        SELECT id, is_public, user_id
        FROM styles 
        WHERE id = $1 AND (
            is_public = true 
            OR user_id = $2
            OR id IN (
                SELECT object_id FROM object_shares
                WHERE object_id = $1 AND object_type = 'style'
                AND (
                    (entity_type = 'user' AND entity_id = $2)
                    OR
                    (entity_type = 'organization' AND entity_id = ANY($3))
                )
            )
        )
        "#,
        style_id,
        authenticated_user_id,
        org_ids_slice
    )
    .fetch_optional(&mut *tx)
    .await;

    let style_info = match style_visibility {
        Err(e) => {
            log::error!("Failed to check style visibility {style_id}: {e}");
            if let Err(rb_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {rb_err}");
            }
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Database error."));
        }
        Ok(None) => {
            // Style doesn't exist or user can't see it (return 404 to not reveal existence)
            if let Err(rb_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {rb_err}");
            }
            return HttpResponse::NotFound()
                .json(ErrorResponse::from("Style not found."));
        }
        Ok(Some(style)) => style,
    };

    // Atomic update with permission check
    let update_result = sqlx::query!(
        r#"
            UPDATE styles 
            SET name = $1, html_url = $2, screenshot_url = $3, user_id = $4, is_public = $5, updated_at = NOW()
        WHERE id = $6 AND (
            user_id = $7 
            OR
            id IN (
                SELECT object_id FROM object_shares
                WHERE object_id = $6 AND object_type = 'style' AND access_level = 'editor'
                AND (
                    (entity_type = 'user' AND entity_id = $7)
                    OR
                    (entity_type = 'organization' AND entity_id = ANY($8))
                )
            )
        )
        "#,
        name,
        html_url.clone(),
        screenshot_url.clone(),
        style_user_id,
        is_public,
        style_id,
        authenticated_user_id,
        org_ids_slice
    )
    .execute(&mut *tx)
    .await;

    match update_result {
        Err(e) => {
            log::error!("Failed to update style {style_id}: {e}");
            if let Err(rb_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {rb_err}");
            }
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to update style."));
        }
        Ok(result) => {
            // Check if any rows were actually updated
            if result.rows_affected() == 0 {
                // Style is visible but user doesn't have edit permission
                if style_info.is_public {
                    // Public style - user can see it but can't edit it (403 Forbidden)
                    log::warn!("User {authenticated_user_id} attempted to update public style {style_id} without permission");
                    if let Err(rb_err) = tx.rollback().await {
                        log::error!("Failed to rollback transaction: {rb_err}");
                    }
                    return HttpResponse::Forbidden()
                        .json(ErrorResponse::from("Access denied."));
                } else {
                    // Private style - this shouldn't happen since we already checked visibility, but return 404 for consistency
                    log::warn!("User {authenticated_user_id} attempted to update private style {style_id} without permission");
                    if let Err(rb_err) = tx.rollback().await {
                        log::error!("Failed to rollback transaction: {rb_err}");
                    }
                    return HttpResponse::NotFound()
                        .json(ErrorResponse::from("Style not found."));
                }
            }
        }
    }

    // Now, fetch the updated data for the response, using the same logic as get_style_by_id
    #[derive(sqlx::FromRow, Debug)]
    struct StyleWithAccessDetails {
        id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        html_url: String,
        screenshot_url: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
        is_public: bool,
    }

    let style_details_result = sqlx::query_as!(
        StyleWithAccessDetails,
        r#"
        WITH ShareAccess AS (
            SELECT object_id, MAX(access_level::text) as access_level
            FROM object_shares
            WHERE object_id = $3 AND object_type = 'style'
              AND ( (entity_type = 'user' AND entity_id = $1) OR (entity_type = 'organization' AND entity_id = ANY($2)) )
            GROUP BY object_id
        )
        SELECT 
            s.id, s.user_id, s.name, s.html_url, s.screenshot_url, 
            s.created_at, s.updated_at, s.is_public, u.email as "creator_email?",
            CASE 
                WHEN s.user_id = $1 THEN 'owner'
                ELSE sa.access_level
            END as "current_user_access_level?"
        FROM styles s
        LEFT JOIN users u ON s.user_id = u.id
        LEFT JOIN ShareAccess sa ON s.id = sa.object_id
        WHERE s.id = $3
        "#,
        authenticated_user_id,
        org_ids_slice,
        style_id
    )
    .fetch_one(&mut *tx)
    .await;

    match style_details_result {
        Ok(details) => {
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction for style update: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::from("Failed to finalize style update."));
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
                    is_public: details.is_public,
                },
                creator_email: details.creator_email,
                current_user_access_level: details.current_user_access_level,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!(
                "Failed to fetch updated style details for style {style_id}: {e}"
            );
            if let Err(rb_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {rb_err}");
            }
            // This case implies the style was deleted between UPDATE and SELECT, or another issue.
            HttpResponse::NotFound()
                .json(ErrorResponse::from("Style not found or access denied."))
        }
    }
}
