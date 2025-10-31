//! Handler for listing user requests.
//!
//! Fetches requests for the authenticated user from the database and returns JSON.

use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Deserialize, Debug, ToSchema)]
pub struct ListRequestsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>, // e.g., created_at, status, what_to_create
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "website clone")]
    pub search: Option<String>, // Search by what_to_create or url? Let's use what_to_create.
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListRequestsResponse {
    pub items: Vec<crate::db::requests::RequestRecord>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/requests",
    responses(
        (status = 200, description = "User requests retrieved successfully", body = ListRequestsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter requests by 'what_to_create' field")
    ),
    tag = "Requests",
    security(("user_auth" = []))
)]
#[actix_web::get("")]
#[instrument(skip(pool, claims, params))]
pub async fn list_requests(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    params: web::Query<ListRequestsParams>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;
    // Validate sort_by against allowed fields
    let sort_by = params
        .sort_by
        .clone()
        .filter(|s| ["created_at", "status", "what_to_create", "finished_at"].contains(&s.as_str()))
        .unwrap_or_else(|| "created_at".into());
    let sort_order = params
        .sort_order
        .clone()
        .filter(|s| s == "asc" || s == "desc")
        .unwrap_or_else(|| "desc".into());
    let search_pattern = params
        .search
        .clone()
        .map(|s| format!("%{s}%"))
        .unwrap_or_else(|| "%".into());
    let search_term = params.search.as_deref().unwrap_or(""); // Use for COALESCE check

    // Total count query
    let total_count = match crate::queries::requests::count_requests_for_user(
        &pool,
        user_id,
        &search_pattern,
        search_term,
    )
    .await
    {
        Ok(count) => count,
        Err(e) => {
            log::error!("Error counting requests for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count requests".into(),
            });
        }
    };

    let items_result = crate::queries::requests::list_requests_for_user(
        &pool,
        user_id,
        &search_pattern,
        search_term,
        &sort_by,
        &sort_order,
        limit,
        offset,
    )
    .await;

    match items_result {
        Ok(items) => HttpResponse::Ok().json(ListRequestsResponse { items, total_count }),
        Err(e) => {
            log::error!("Error fetching requests for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch requests".into(),
            })
        }
    }
}
