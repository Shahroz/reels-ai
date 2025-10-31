//! Handler for creating a new creative.
//!
//! POST /api/creatives
//! Requires authentication and validates collection ownership.
// Declare the new requests submodule

use crate::auth::tokens::Claims;
use crate::routes::creatives::create_creative_request::CreateCreativeRequest;
use crate::routes::creatives::responses::CreativeResponse;
use crate::routes::error_response::ErrorResponse;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/creatives",
    request_body = CreateCreativeRequest,
    responses(
        (status = 201, description = "Created", body = CreativeResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security( // Add security requirement
        ("bearer_auth" = [])
    )
)]
#[post("")]
#[instrument(skip(pool, payload, auth))]
pub async fn create_creative(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<CreateCreativeRequest>,
    auth: Claims, // Add Claims argument
) -> impl Responder {
    // 1. Verify collection ownership
    let collection_check = sqlx::query!(
        "SELECT id FROM collections WHERE id = $1 AND user_id = $2",
        payload.collection_id,
        auth.user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    if let Err(e) = collection_check {
        log::error!("DB error checking collection ownership: {e:?}");
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to verify collection access".to_string(),
        });
    }
    if collection_check.unwrap().is_none() {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Access denied to the specified collection".to_string(),
        });
    }

    #[derive(sqlx::FromRow, Debug)]
    struct CreativeWithAccessDetails {
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

    // Insert using fields: name, html_url, bundle_id, and screenshot_url, and return creator_email & access level
    let result = sqlx::query_as!(
        CreativeWithAccessDetails,
        r#"
        WITH inserted_creative AS (
            INSERT INTO creatives (
                name,
                collection_id,
                creative_format_id,
                style_id,
                document_ids,
                asset_ids,
                html_url,
                draft_url,
                bundle_id,
                screenshot_url,
                is_published,
                publish_url,
                created_at,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                NOW(), NOW()
            )
            RETURNING
                id,
                name,
                collection_id,
                creative_format_id,
                style_id,
                document_ids,
                asset_ids,
                html_url,
                draft_url,
                bundle_id,
                screenshot_url,
                is_published,
                publish_url,
                created_at,
                updated_at,
                $13::UUID AS owner_user_id
        )
        SELECT
            ic.id,
            ic.name,
            ic.collection_id,
            ic.creative_format_id,
            ic.style_id,
            ic.document_ids,
            ic.asset_ids,
            ic.html_url,
            ic.draft_url,
            ic.bundle_id,
            ic.screenshot_url,
            ic.is_published,
            ic.publish_url,
            ic.created_at,
            ic.updated_at,
            u.email AS "creator_email?",
            'owner'::TEXT AS "current_user_access_level?"
        FROM inserted_creative ic
        JOIN users u ON u.id = ic.owner_user_id
        "#,
        payload.name,
        payload.collection_id,
        payload.creative_format_id,
        payload.style_id,
        payload.document_ids.as_ref().map(|v| v.as_slice()),
        payload.asset_ids.as_ref().map(|v| v.as_slice()),
        payload.html_url,
        None::<String>,               // draft_url
        None::<sqlx::types::Uuid>,    // bundle_id (if not provided in request)
        payload.screenshot_url,
        false,                        // is_published
        None::<String>,               // publish_url
        auth.user_id                  // owner_user_id, used for fetching creator_email
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(details) => {
            let response = CreativeResponse {
                creative: crate::db::creatives::Creative {
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
                    updated_at: details.updated_at
                },
                creator_email: details.creator_email,
                current_user_access_level: details.current_user_access_level,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!("DB error inserting creative: {e:?}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create creative".to_string(),
            })
        }
    }
}
