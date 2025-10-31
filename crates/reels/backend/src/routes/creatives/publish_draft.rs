//! Handles the request to publish a draft creative.
//!
//! This endpoint takes a creative ID, fetches its draft content,
//! uploads it as the main creative HTML, regenerates the screenshot,
//! and updates the database.

//! - 2025-05-20T18:51:48Z @AI: Initial implementation.

use crate::db::creatives::Creative; // For utoipa response body
use crate::routes::creatives::creative_asset_utils::upload_creative_assets;
use crate::routes::creatives::responses::CreativeResponse;
use crate::routes::error_response::ErrorResponse; // For utoipa response body
use crate::services::gcs::parse_gcs_url::parse_gcs_url;
use crate::queries::organizations::find_active_memberships_for_user;
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

// Note: This function will likely exceed the 50 LoC guideline due to multiple sequential I/O operations
// (DB fetch, HTTP GET, GCS uploads x2, Zyte API call, DB update). This is justified by the nature
// of the operation which involves a pipeline of dependent steps.
#[utoipa::path(
    post,
    path = "/api/creatives/{id}/publish_draft", // Full path in API, assuming /creatives scope
    tag = "creatives",
    params(
        ("id" = uuid::Uuid, Path, description = "Creative ID")
    ),
    responses(
        (status = 200, description = "Creative draft published successfully", body = CreativeResponse),
        (status = 400, description = "Bad request (e.g., creative does not have a draft URL)", body = ErrorResponse),
        (status = 404, description = "Creative not found or not owned by user", body = ErrorResponse),
        (status = 500, description = "Server error during draft publishing process", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(
    skip(path, user_claims, pool, gcs_client),
    fields(creative_id = %path.as_ref(), user_id = %user_claims.user_id)
)]
#[actix_web::post("/{id}/publish_draft")]
pub async fn publish_draft(
    path: actix_web::web::Path<uuid::Uuid>,
    user_claims: crate::auth::tokens::Claims,
    pool: actix_web::web::Data<sqlx::PgPool>,
    gcs_client: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    let creative_id = path.into_inner();
    let user_id = user_claims.user_id;

    // --- Organization and Sharing Setup ---
    let org_memberships = match find_active_memberships_for_user(&pool, user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            tracing::error!("Failed to fetch organization memberships for user {}: {}", user_id, e);
            return Ok(actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve necessary user data.".to_string(),
                },
            ));
        }
    };
    let org_ids: Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    #[derive(sqlx::FromRow, Debug)]
    struct CreativeForPublish {
        draft_url: Option<String>,
    }

   // 1. Fetch the Creative record and verify ownership or 'editor' permission.
   let creative = match sqlx::query_as!(
       CreativeForPublish,
       r#"
        WITH CreativeShares_CTE AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'creative'
              AND os.object_id = $1
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $2)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3))
                )
        ),
        CollectionShares_CTE AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            INNER JOIN creatives c ON c.collection_id = os.object_id
            WHERE c.id = $1
              AND os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $2)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3))
                )
        )
        SELECT 
            c.draft_url
          FROM creatives c 
          INNER JOIN collections col ON c.collection_id = col.id
          WHERE c.id = $1 AND (
            col.user_id = $2 
            OR EXISTS (SELECT 1 FROM CreativeShares_CTE WHERE rn = 1 AND access_level = 'editor')
            OR EXISTS (SELECT 1 FROM CollectionShares_CTE WHERE rn = 1 AND access_level = 'editor')
          )
       "#,
       creative_id,
       user_id,
       &org_ids
   )
    .fetch_one(&**pool)
    .await
    {
        Ok(c) => c,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: format!("Creative with ID {creative_id} not found or you don't have permission to edit."),
                },
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch creative: {:?}", e);
            return Ok(actix_web::HttpResponse::InternalServerError()
                .json(crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve creative details.".to_string(),
                }));
        }
    };

    // 2. Check if draft_url exists.
    let draft_url_str: String = match creative.draft_url {
        Some(url) => url,
        None => {
            return Ok(actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Creative does not have a draft URL to publish.".to_string(),
                },
            ));
        }
    };

    // 3. Read the content of draft.html from creative.draft_url using GCS client instead of reqwest
    let draft_content_bytes = match parse_gcs_url(&draft_url_str) {
        Ok((bucket_name, object_name)) => {
            match gcs_client.get_ref().as_ref().download_object_as_string(&bucket_name, &object_name).await {
                Ok(content) => content.into_bytes(),
                Err(e) => {
                    tracing::error!("Failed to download draft content from GCS bucket '{}', object '{}': {:?}", bucket_name, object_name, e);
                    return Ok(actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: "Failed to read content from draft storage.".to_string(),
                        },
                    ));
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to parse draft URL '{}': {}", draft_url_str, e);
            return Ok(actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Invalid draft URL format.".to_string(),
                },
            ));
        }
    };

    // 4. Upload draft content as new main HTML and generate new screenshot using the utility function.
    // The utility function handles GCS bucket name, Zyte key, and standardized GCS paths.
    let (new_html_url, new_screenshot_url) = match upload_creative_assets(
        gcs_client.get_ref().as_ref(),
        creative_id,
        draft_content_bytes.to_vec(),
    )
    .await
    {
        Ok(urls) => urls,
        Err(e) => {
            return Ok(actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: format!("Failed to process and upload draft assets: {e}"),
                },
            ));
        }
    };

    // 5. Update the database: set new html_url, new screenshot_url, draft_url = NULL, and updated_at = NOW().

    #[derive(sqlx::FromRow, Debug)]
    struct UpdatedCreativeDetails {
        id: Option<Uuid>,
        name: Option<String>,
        collection_id: Option<Uuid>,
        creative_format_id: Option<Uuid>,
        style_id: Option<Uuid>,
        document_ids: Option<Vec<Uuid>>,
        asset_ids: Option<Vec<Uuid>>,
        html_url: Option<String>,
        draft_url: Option<String>,
        bundle_id: Option<Uuid>,
        screenshot_url: Option<String>,
        is_published: Option<bool>,
        publish_url: Option<String>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
    }

    let updated_creative_details = match sqlx::query_as!(
        UpdatedCreativeDetails,
        r#"
        WITH updated_creative AS (
            UPDATE creatives
            SET
                html_url = $2,
                screenshot_url = $3,
                draft_url = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        )
        SELECT 
            uc.id, uc.name, uc.collection_id, uc.creative_format_id::uuid, uc.style_id, uc.document_ids, 
            uc.asset_ids, uc.html_url, uc.draft_url, uc.screenshot_url, uc.is_published, uc.publish_url, 
            uc.created_at, uc.updated_at, uc.bundle_id,
            u.email AS "creator_email?",
            'owner'::TEXT AS "current_user_access_level?"
        FROM updated_creative uc
        LEFT JOIN collections col ON uc.collection_id = col.id
        LEFT JOIN users u ON col.user_id = u.id
        "#,
        creative_id,
        new_html_url,
        new_screenshot_url
    )
    .fetch_one(&**pool)
    .await
    {
        Ok(uc) => uc,
        Err(e) => {
            tracing::error!("Failed to update creative in DB after publishing draft: {:?}", e);
            return Ok(actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to finalize creative update in database.".to_string(),
                },
            ));
        }
    };

    let response = CreativeResponse {
        creative: Creative {
            id: updated_creative_details.id.unwrap_or_else(|| {
                tracing::error!("id is None after publishing draft");
                uuid::Uuid::nil()
            }),
            name: updated_creative_details.name.unwrap_or_else(|| {
                tracing::error!("name is None after publishing draft");
                "Unknown Creative".to_string()
            }),
            collection_id: updated_creative_details.collection_id,
            creative_format_id: updated_creative_details.creative_format_id.unwrap_or_else(|| {
                tracing::error!("creative_format_id is None after publishing draft");
                uuid::Uuid::nil()
            }),
            style_id: updated_creative_details.style_id,
            document_ids: updated_creative_details.document_ids,
            asset_ids: updated_creative_details.asset_ids,
            html_url: updated_creative_details.html_url.unwrap_or_else(|| {
                tracing::error!("html_url is None after publishing draft");
                String::new()
            }),
            draft_url: updated_creative_details.draft_url,
            bundle_id: updated_creative_details.bundle_id,
            screenshot_url: updated_creative_details.screenshot_url.unwrap_or_else(|| {
                tracing::error!("screenshot_url is None after publishing draft");
                String::new()
            }),
            is_published: updated_creative_details.is_published.unwrap_or(false),
            publish_url: updated_creative_details.publish_url,
            created_at: updated_creative_details.created_at.unwrap_or_else(|| {
                tracing::error!("created_at is None after publishing draft");
                chrono::Utc::now()
            }),
            updated_at: updated_creative_details.updated_at.unwrap_or_else(|| {
                tracing::error!("updated_at is None after publishing draft");
                chrono::Utc::now()
            }),
        },
        creator_email: updated_creative_details.creator_email,
        current_user_access_level: updated_creative_details.current_user_access_level,
    };

    // 6. Return the updated Creative record.
    Ok(actix_web::HttpResponse::Ok().json(response))
}
