//! Handler for duplicating a creative.
//!
//! POST /api/creatives/{id}/duplicate
//! Creates a new creative as a copy of an existing one, owned by the requester.

use crate::auth::tokens::Claims;
use crate::queries::creatives::get_creative_details::get_creative_details;
use crate::routes::error_response::ErrorResponse;
use crate::routes::creatives::responses::GetCreativeDetails;
use crate::services::gcs::gcs_operations::GCSOperations;
use actix_web::{post, web, HttpResponse, Responder};
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/creatives/{id}/duplicate",
    params(
        ("id" = Uuid, Path, description = "ID of the creative to duplicate")
    ),
    responses(
        (status = 201, description = "Creative duplicated successfully", body = GetCreativeDetails),
        (status = 400, description = "Bad Request (e.g., user has no collection to duplicate into)", body = ErrorResponse),
        (status = 403, description = "Forbidden - User does not have access to the original creative", body = ErrorResponse),
        (status = 404, description = "Original creative not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/{id}/duplicate")]
#[instrument(skip(pool, gcs_client, claims))]
pub async fn duplicate_creative(
    pool: web::Data<PgPool>,
    gcs_client: web::Data<Arc<dyn GCSOperations>>,
    id: web::Path<Uuid>,
    claims: Claims,
) -> impl Responder {
    let user_id = claims.user_id;
    let original_creative_id = *id;

    // 1. Fetch original creative and verify access
    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(pool.get_ref(), user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!("Failed to fetch organization memberships for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve user data"));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    let original_creative = match sqlx::query_as!(
        crate::db::creatives::Creative,
        r#"
        WITH CreativeShares_CTE AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'creative' AND os.object_id = $2
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
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
            WHERE c.id = $2 AND os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                )
        )
        SELECT c.*
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        WHERE c.id = $2 AND (
            col.user_id = $1 
            OR EXISTS (SELECT 1 FROM CreativeShares_CTE WHERE rn = 1)
            OR EXISTS (SELECT 1 FROM CollectionShares_CTE WHERE rn = 1)
        )
        "#,
        user_id,
        original_creative_id,
        &org_ids
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(creative)) => creative,
        Ok(None) => {
            log::warn!("Attempt to duplicate non-existent or inaccessible creative {original_creative_id} by user {user_id}");
            return HttpResponse::NotFound().json(ErrorResponse::from("Creative not found or not accessible."));
        },
        Err(e) => {
            log::error!("DB error fetching creative for duplication: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to fetch original creative."));
        }
    };

    // 2. Determine collection for the new creative. It must be owned by the current user.
    let new_collection_id = if let Some(cid) = original_creative.collection_id {
        let is_owner = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)",
            cid,
            user_id
        )
       .fetch_one(pool.get_ref())
       .await
       .ok()
       .flatten();

       if is_owner.unwrap_or(false) {
           Some(cid)
        } else {
            sqlx::query_scalar!("SELECT id FROM collections WHERE user_id = $1 LIMIT 1", user_id)
                .fetch_optional(pool.get_ref()).await.unwrap_or(None)
        }
    } else {
        sqlx::query_scalar!("SELECT id FROM collections WHERE user_id = $1 LIMIT 1", user_id)
            .fetch_optional(pool.get_ref()).await.unwrap_or(None)
    };

    let new_collection_id = match new_collection_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(ErrorResponse::from("You must have at least one collection to duplicate a creative into.")),
    };


    // 3. Duplicate assets in GCS
    let new_creative_id = Uuid::new_v4();
    let bucket_name = match std::env::var("GCS_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            log::error!("GCS_BUCKET environment variable not set.");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Server configuration error."));
        }
    };

    let original_object_name = format!("creatives/{original_creative_id}/creative.html");

    let original_html_bytes = match gcs_client.download_object_as_string(&bucket_name, &original_object_name).await {
        Ok(s) => s.into_bytes(),
        Err(e) => {
            log::error!("Failed to download original creative HTML from GCS: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to access original creative content."));
        }
   };

   let (new_html_url, new_screenshot_url) = match crate::routes::creatives::creative_asset_utils::upload_creative_assets(
        gcs_client.get_ref().as_ref(),
       new_creative_id,
       original_html_bytes,
   ).await {
        Ok((html_url, screenshot_url)) => (html_url, screenshot_url),
        Err(e) => {
            log::error!("Failed to upload duplicated creative assets: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to save new creative assets."));
        }
    };

    // 4. Insert new creative record into the database
    let re = Regex::new(r"^(.*) \(COPY(?: (\d+))?\)$").unwrap();
    let new_name = if let Some(caps) = re.captures(&original_creative.name) {
        let base_name = caps.get(1).map_or(original_creative.name.as_str(), |m| m.as_str());
        if let Some(num_match) = caps.get(2) {
            let num: u32 = num_match.as_str().parse().unwrap_or(1);
            format!("{} (COPY {})", base_name, num + 1)
        } else {
            format!("{base_name} (COPY 2)")
        }
    } else {
        format!("{} (COPY)", original_creative.name)
    };

    let insert_result = sqlx::query!(
        r#"
        INSERT INTO creatives (
            id, name, collection_id, creative_format_id, style_id, document_ids,
            asset_ids, html_url, draft_url, bundle_id, screenshot_url,
            is_published, publish_url
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, false, NULL
        )
        "#,
        new_creative_id,
        new_name,
        Some(new_collection_id),
        original_creative.creative_format_id,
        original_creative.style_id,
        original_creative.document_ids.as_deref(),
        original_creative.asset_ids.as_deref(),
        new_html_url,
        Some(new_html_url.clone()), // draft_url
        original_creative.bundle_id,
        new_screenshot_url
    )
    .execute(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        log::error!("DB error inserting duplicated creative: {e}");
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to save duplicated creative."));
    }

    match get_creative_details(pool.get_ref(), user_id, new_creative_id).await {
        Ok(Some(item)) => HttpResponse::Created().json(item),
        Ok(None) => {
            log::error!("Could not fetch newly created creative {new_creative_id} right after duplication");
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve duplicated creative after creation."))
        },
        Err(e) => {
            log::error!("DB error fetching details of duplicated creative: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to save duplicated creative."))
        }
    }
}
