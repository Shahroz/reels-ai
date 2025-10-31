//! Defines handler for listing styles with pagination, sorting, and searching.
//!
//! Handles GET requests to `/api/styles`. Retrieves a paginated, sorted, and filtered
//! list of styles for the authenticated user, returning both items and total count.

use crate::auth::tokens::Claims;
use crate::db;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::queries::styles::{
    count_styles_for_user::count_styles_for_user, list_styles_for_user::list_styles_for_user,
};
use crate::routes::error_response::ErrorResponse;
use crate::routes::styles::responses::StyleResponseWithFavorite;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct ListStylesParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_public: Option<bool>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListStylesResponse {
    pub items: Vec<StyleResponseWithFavorite>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/styles",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (id, name, created_at, updated_at - default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter styles by name"),
        ("is_favorite" = Option<bool>, Query, description = "Filter by favorite status (true for favorited, false for non-favorited)"),
        ("is_public" = Option<bool>, Query, description = "Filter by visibility (true for public styles, false for private styles, null for both)")
    ),
    responses(
        (status = 200, description = "List styles", body = ListStylesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Styles",
    security(("user_auth" = []))
)]
#[get("")]
#[instrument(skip(pool, claims, params), fields(user_id = %claims.user_id))]
pub async fn list_styles(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<ListStylesParams>,
) -> impl Responder {
    let user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    let raw_sort_by = params.sort_by.clone().unwrap_or_else(|| "created_at".into());
    let raw_sort_order = params.sort_order.clone().unwrap_or_else(|| "desc".into());

    let search_pattern = params
        .search
        .clone()
        .map(|s| format!("%{}%", s.replace("%", "\\%").replace("_", "\\_")))
        .unwrap_or_else(|| "%".into());

    let is_favorite_filter = params.is_favorite;
    let is_public_filter = params.is_public;

    let org_memberships = match find_active_memberships_for_user(&pool, user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!("Error fetching organization memberships for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user organization memberships".into(),
            });
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    // --- Total count query ---
    let total_count = match count_styles_for_user(&pool, user_id, &search_pattern, &org_ids, is_favorite_filter, is_public_filter).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("Failed to count styles for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count styles".into(),
            });
        }
    };

    // --- Items query ---
    let items = match list_styles_for_user(&pool, user_id, &search_pattern, &org_ids, limit, offset, &raw_sort_by, &raw_sort_order, is_favorite_filter, is_public_filter).await {
        Ok(list) => list,
        Err(e) => {
            log::error!("Failed to fetch styles for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch styles".into(),
            });
        }
    };

    let response_items = items
        .into_iter()
        .map(|item| StyleResponseWithFavorite {
            style: db::styles::Style {
                id: item.id,
                user_id: item.user_id,
                name: item.name,
                html_url: item.html_url,
                screenshot_url: item.screenshot_url,
                is_public: item.is_public,
                created_at: item.created_at,
                updated_at: item.updated_at,
            },
            creator_email: item.creator_email,
            current_user_access_level: item.current_user_access_level,
            is_favorite: item.is_favorite.unwrap_or(false),
        })
        .collect();

    HttpResponse::Ok().json(ListStylesResponse {
        items: response_items,
        total_count,
    })
}
