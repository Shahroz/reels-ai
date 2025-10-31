//! Provides an endpoint to fetch the HTML content of a creative, with a <base> tag injected.
//!
//! The endpoint `GET /api/creatives/{id}/content` retrieves a creative's HTML,
//! determines the correct GCS base path for its assets, injects a `<base>` tag
//! into the HTML's `<head>` section, and returns the modified HTML.
//! This allows relative paths in the creative's HTML to resolve correctly when served.
//! Returns a JSON object containing the HTML content and a boolean indicating if it's a draft.
//! Adheres to `rust_guidelines.md`: file-level documentation, one public function, in-file tests.

use crate::routes::creatives::get_creative_content_response::GetCreativeContentResponse;
use crate::utils::extract_html_colors::extract_colors_from_html;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::services::gcs::parse_gcs_url::parse_gcs_url;

#[utoipa::path(
    get,
    path = "/api/creatives/{id}/content",
    tag = "Creatives",
    params(
        ("id" = sqlx::types::Uuid, Path, description = "Creative ID")
    ),
    responses(
        (status = 200, description = "JSON object with HTML content and draft status", body = GetCreativeContentResponse, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Creative not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(pool, gcs_client))]
#[actix_web::get("/{creative_id}/content")]
pub async fn get_creative_content(
    path: actix_web::web::Path<sqlx::types::Uuid>,
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    gcs_client: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
) -> impl actix_web::Responder {
    let creative_id = path.into_inner();
    let user_id = claims.user_id;

    log::info!("logremove - get_creative_content called for creative_id: {}, user_id: {}", creative_id, user_id);

    // Fetch user's organization memberships
    let org_memberships = match find_active_memberships_for_user(pool.get_ref(), user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!(
                "logremove - Failed to fetch organization memberships for user {user_id} while fetching creative content {creative_id}: {e}"
            );
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve necessary user data".to_string(),
                }
            );
        }
    };
    let org_ids: Vec<sqlx::types::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    log::info!("logremove - User {} belongs to org_ids: {:?}", user_id, org_ids);

    #[derive(sqlx::FromRow, Debug)]
    struct CreativeContent {
        html_url: String,
        draft_url: Option<String>,
        current_user_access_level: Option<String>,
    }

    // Fetch creative and verify ownership or shared access
    let creative_fetch_result = sqlx::query_as!(
        CreativeContent,
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'
        ),
        RankedShares_CTE AS (
            SELECT
                os.object_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            WHERE (
                    (os.object_type = 'creative' AND os.object_id = $2)
                    OR 
                    (os.object_type = 'collection' AND os.object_id = (SELECT collection_id FROM creatives WHERE id = $2))
                )
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3))
                )
        ),
        EffectiveShares_CTE AS (
            SELECT object_id, access_level FROM RankedShares_CTE WHERE rn = 1
        )
        SELECT 
            cr.html_url,
            cr.draft_url,
            CASE
                WHEN col.user_id = $1 THEN 'owner'::TEXT
                ELSE COALESCE(
                    creative_share.access_level::TEXT,
                    collection_share.access_level::TEXT
                )
            END AS current_user_access_level
        FROM creatives cr
        INNER JOIN collections col ON cr.collection_id = col.id
        LEFT JOIN EffectiveShares_CTE creative_share ON cr.id = creative_share.object_id
        LEFT JOIN EffectiveShares_CTE collection_share ON col.id = collection_share.object_id
        WHERE cr.id = $2 AND (col.user_id = $1 OR creative_share.access_level IS NOT NULL OR collection_share.access_level IS NOT NULL)
        "#,
        user_id,
        creative_id,
        &org_ids
    )
    .fetch_optional(pool.get_ref())
    .await;

    let creative = match creative_fetch_result {
        Ok(Some(c)) => {
            if c.current_user_access_level.is_some() {
                log::info!("logremove - Creative {} found and authorized for user {} with access level: {:?}", creative_id, user_id, c.current_user_access_level);
                c
            } else {
                log::warn!("logremove - Creative {creative_id} found BUT user {user_id} NOT authorized (access_level is None).");
                return actix_web::HttpResponse::Forbidden().json(
                    crate::routes::error_response::ErrorResponse { 
                        error: "Access to creative content forbidden".to_string(),
                    }
                );
            }
        },
        Ok(None) => {
            log::warn!("logremove - Creative not found or user not authorized: creative_id={creative_id}, user_id={user_id}");
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse { 
                    error: "Creative content not found or not authorized".to_string(),
                }
            );
        }
        Err(e) => {
            log::error!("logremove - Database error fetching creative content for {creative_id}: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse { 
                    error: "Database error".to_string(),
                }
            );
        }
    };

    log::info!("logremove - Proceeding to fetch GCS content for creative: {}", creative_id);

    let (source_html_url_str, is_draft) =
        if let Some(draft_url_val) = &creative.draft_url {
            (draft_url_val.as_str(), true)
        } else {
            (creative.html_url.as_str(), false)
        };

    let (bucket_name, object_name) = match parse_gcs_url(source_html_url_str) {
        Ok((b, o)) => (b, o),
        Err(e) => {
            tracing::error!("Failed to parse GCS URL '{}': {}", source_html_url_str, e);
            return actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Invalid creative content URL format".to_string(),
                },
            );
        }
    };

    // 1. Fetch HTML content using GCSClient
    //    Note: gcs_client is actix_web::web::Data<Arc<dyn GCSOperations>>, so use .get_ref().as_ref() to access GCSOperations methods.
    let html_string = match gcs_client.get_ref().as_ref().download_object_as_string(&bucket_name, &object_name).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to download HTML from GCS bucket '{}', object '{}': {:?}", bucket_name, object_name, e);
            log::error!("logremove - Failed to download HTML from GCS for creative {creative_id}: bucket={bucket_name}, object={object_name}, error={e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve creative HTML from storage".to_string(),
                },
            );
        }
    };
    
    let colors = extract_colors_from_html(&html_string);
    
    // 2. Construct the response object
    let response_data = crate::routes::creatives::get_creative_content_response::GetCreativeContentResponse {
        html_content: html_string,
        is_draft,
       extracted_colors: Some(colors),
    };

    log::info!("logremove - Successfully prepared content for creative: {}. is_draft: {}", creative_id, is_draft);
    actix_web::HttpResponse::Ok().json(response_data)
}
