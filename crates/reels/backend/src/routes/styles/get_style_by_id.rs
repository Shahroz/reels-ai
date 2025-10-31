//! Defines the handler for retrieving a specific style by its ID.
//!
//! This function handles GET requests to `/api/styles/{id}`. It fetches a single
//! style identified by the path parameter `id`, ensuring it belongs to the
//! authenticated user or is shared with them, or is public. Returns the Style object with access
//! level information or appropriate error responses.
//! Requires DB pool, path extractor, and user claims.

use crate::auth::tokens::Claims;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::error_response::ErrorResponse;
use crate::routes::styles::responses::StyleResponseWithFavorite;
use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/styles/{id}",
    params(("id" = String, Path, description = "Style ID")),
    responses(
        (status = 200, description = "Get style", body = StyleResponseWithFavorite),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Style not found or access denied", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[actix_web::get("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn get_style_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let style_id = path.into_inner();

    let org_memberships = match find_active_memberships_for_user(&pool, authenticated_user_id).await
    {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!(
                "Error fetching organization memberships for user {authenticated_user_id}: {e}"
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user organization memberships for permission check"
                    .into(),
            });
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice = if org_ids.is_empty() { &[] } else { &org_ids[..] };

    #[derive(sqlx::FromRow, Debug)]
    struct StyleWithAccessDetails {
        id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        html_url: String,
        screenshot_url: String,
        is_public: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        creator_email: Option<String>,
        current_user_access_level: Option<String>,
        is_favorite: Option<bool>,
    }

    let style_result = sqlx::query_as!(
        StyleWithAccessDetails,
        r#"
        WITH ShareAccess AS (
            SELECT 
                object_id,
                -- Prioritize 'editor' over 'viewer' if user has multiple shares
                MAX(CASE WHEN access_level::text = 'editor' THEN 2 WHEN access_level::text = 'viewer' THEN 1 ELSE 0 END) as priority,
                MAX(access_level::text) as access_level
            FROM object_shares
            WHERE object_id = $3
              AND object_type = 'style'
              AND (
                  (entity_type = 'user' AND entity_id = $1)
                  OR 
                  (entity_type = 'organization' AND entity_id = ANY($2))
              )
            GROUP BY object_id
        )
        SELECT 
            s.id, s.user_id, s.name, s.html_url, s.screenshot_url, s.is_public,
            s.created_at, s.updated_at, u.email as "creator_email?",
            CASE 
                WHEN s.user_id = $1 THEN 'owner'
                ELSE sa.access_level
            END as "current_user_access_level?",
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $1 AND entity_id = s.id AND entity_type = 'style')), false) AS is_favorite
        FROM styles s
        LEFT JOIN users u ON s.user_id = u.id
        LEFT JOIN ShareAccess sa ON s.id = sa.object_id
        WHERE s.id = $3 AND (s.user_id = $1 OR s.is_public = true OR sa.object_id IS NOT NULL)
        "#,
        authenticated_user_id,
        org_ids_slice,
        style_id
    )
    .fetch_optional(&**pool)
    .await;

    match style_result {
        Ok(Some(details)) => {
            let response = StyleResponseWithFavorite {
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
                is_favorite: details.is_favorite.unwrap_or(false),
            };
            HttpResponse::Ok().json(response)
        }
        Ok(None) => {
            log::warn!(
                "User {authenticated_user_id} attempted to access non-existent or forbidden style {style_id}"
            );
            HttpResponse::NotFound().json(ErrorResponse::from("Style not found or access denied."))
        }
        Err(e) => {
            log::error!("Error retrieving style {style_id}: {e}");
            HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to retrieve style."))
        }
    }
}
