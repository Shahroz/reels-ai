//! Handles the discarding of a creative draft.
//!
//! This endpoint allows an authenticated user to discard a previously saved draft
//! for one of their creatives. It involves deleting the draft from GCS (if a
//! draft URL exists) and clearing the `draft_url` field in the database.
//! The function `discard_draft` implements this logic. It first fetches the creative
//! and verifies ownership. Then, it attempts to delete any associated GCS object
//! for the draft. Finally, it updates the creative's record in the database,
//! setting `draft_url` to NULL and updating `updated_at`.

use actix_web::{web, HttpResponse, Responder};
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};

// Assuming Creative struct is defined in crate::db::creatives
// and has fields: id, user_id (or similar for ownership), draft_url, updated_at
// and derives sqlx::FromRow, serde::Serialize, utoipa::ToSchema
use crate::db::creatives::Creative;
use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::routes::creatives::responses::CreativeResponse;

const GCS_DRAFT_BUCKET_ENV_VAR: &str = "GCS_DRAFT_BUCKET";

// Helper struct to include owner_user_id from the joined collections table
#[derive(sqlx::FromRow, Debug)]
struct CreativeWithOwnership {
    draft_url: Option<String>, // This is the one we care about for GCS deletion
    owner_user_id: Option<Uuid>, // This comes from `collections.user_id`
}

#[utoipa::path(
    post,
    path = "/api/creatives/{id}/discard-draft",
    tag = "Creatives",
    params(
        ("id" = Uuid, Path, description = "Creative ID")
    ),
    responses(
        (status = 200, description = "Draft discarded successfully", body = CreativeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - User does not own this creative", body = ErrorResponse),
        (status = 404, description = "Creative not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{id}/discard-draft")]
pub async fn discard_draft(
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
    pool: web::Data<sqlx::PgPool>,
    gcs_client: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
) -> impl Responder {
    let creative_id = path.into_inner();
    let user_id = claims.user_id; // Assuming claims.sub is the user_id (Uuid)

    // 1. Fetch Creative and verify ownership
    let creative_with_owner = match sqlx::query_as!(
        CreativeWithOwnership,
        r#"SELECT
                c.draft_url,
                col.user_id AS "owner_user_id?"
            FROM creatives c
            LEFT JOIN collections col ON c.collection_id = col.id
            WHERE c.id = $1"#,
        creative_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(cr) => {
            // Verify ownership:
            // A creative is owned if its collection_id is not NULL,
            // the collection exists, and collection.user_id matches claims.sub.
            // If creative_with_owner.owner_user_id is None, it means either collection_id was NULL,
            // or the collection didn't exist (which implies collection_id was NULL or invalid),
            // or collection.user_id was NULL (user_id on collections table is NOT NULL, so this shouldn't happen if join succeeds).
            // So, if owner_user_id is None, or if it's Some(id) where id != user_id, then it's forbidden.
            if cr.owner_user_id != Some(user_id) {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Forbidden".to_string(),
                });
            }
            cr
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Not Found".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Failed to fetch creative: {:?}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal Server Error".to_string(),
            });
        }
    };

    // 2. GCS Draft Deletion (Optional but Recommended)
    if let Some(draft_url_str) = &creative_with_owner.draft_url {
        match std::env::var(GCS_DRAFT_BUCKET_ENV_VAR) {
            Ok(bucket_name) => {
               match url::Url::parse(draft_url_str) {
                   Ok(parsed_url) => {
                        let object_path_segments: std::vec::Vec<&str> = parsed_url.path_segments().map_or_else(std::vec::Vec::new, |segments| segments.collect());
                       if object_path_segments.len() > 1 && object_path_segments[0] == bucket_name {
                           let object_name = object_path_segments[1..].join("/");
                           if !object_name.is_empty() {
                                tracing::info!("Attempting to delete GCS object: gs://{}/{}", bucket_name, object_name);
                                if let Err(e) = gcs_client.delete_object(&bucket_name, &object_name).await {
                                    tracing::error!("Failed to delete GCS object '{}': {:?}. Proceeding with DB update.", object_name, e);
                                }
                            } else {
                                tracing::warn!("Empty object name derived from draft URL: {}. Skipping GCS deletion.", draft_url_str);
                            }
                        } else {
                             tracing::warn!("Draft URL '{}' does not match expected GCS path structure for bucket '{}'. Skipping GCS deletion.", draft_url_str, bucket_name);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse draft_url '{}': {:?}. Skipping GCS deletion.", draft_url_str, e);
                    }
                }
            }
            Err(_) => {
                tracing::warn!("GCS_DRAFT_BUCKET environment variable not set. Skipping GCS deletion for draft URL: {}.", draft_url_str);
            }
        }
    }

    #[derive(sqlx::FromRow, Debug)]
    struct DiscardDraftResponse {
        id: Uuid,
        name: String,
        collection_id: Option<Uuid>,
        creative_format_id: Uuid,
        style_id: Option<Uuid>,
        document_ids: Option<Vec<Uuid>>,
        asset_ids: Option<Vec<Uuid>>,
        html_url: String,
        draft_url: Option<String>,
        bundle_id: Option<Uuid>,
        screenshot_url: String,
        is_published: bool,
        publish_url: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
    }

    // 3. Database Update
    match sqlx::query_as!(
        DiscardDraftResponse,
        r#"UPDATE creatives
            SET draft_url = NULL, updated_at = NOW()
            WHERE id = $1
            RETURNING
                creatives.id, -- Qualify with table name for clarity
                creatives.name,
                creatives.collection_id,
                creatives.creative_format_id,
                creatives.style_id,
                creatives.document_ids,
                creatives.asset_ids,
                creatives.html_url,
                creatives.draft_url, -- will be NULL
                creatives.bundle_id,
                creatives.screenshot_url,
                creatives.is_published,
                creatives.publish_url,
                creatives.created_at,
                creatives.updated_at,
                (SELECT u.email FROM users u JOIN collections col ON u.id = col.user_id WHERE col.id = creatives.collection_id) AS creator_email,
                'owner'::TEXT AS current_user_access_level
        "#,
        creative_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(details) => {
            let response = CreativeResponse {
                creative: Creative {
                    id: details.id,
                    name: details.name,
                    collection_id: details.collection_id,
                    creative_format_id: details.creative_format_id,
                    style_id: details.style_id,
                    document_ids: details.document_ids,
                    asset_ids: details.asset_ids,
                    html_url: details.html_url,
                    draft_url: details.draft_url,
                    bundle_id: details.bundle_id,
                    screenshot_url: details.screenshot_url,
                    is_published: details.is_published,
                    publish_url: details.publish_url,
                    created_at: details.created_at,
                    updated_at: details.updated_at,
                },
                creator_email: details.creator_email,
                current_user_access_level: details.current_user_access_level,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("Failed to update creative in database: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal Server Error".to_string(),
            })
        }
    }
}
