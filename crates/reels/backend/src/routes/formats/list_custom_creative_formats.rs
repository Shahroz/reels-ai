//! Handler for listing custom creative formats for the authenticated user.
//!
//! GET /api/custom-creative-formats

//! Fetches custom creative formats defined by the logged-in user.
//! Requires JWT authentication to identify the user.
//! Supports potential future pagination/filtering (not implemented yet).

use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::queries::custom_creative_formats::{count::count_formats, list::list_formats};
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid; // Import Uuid

#[derive(Deserialize, Debug, ToSchema)]
pub struct ListCustomCreativeFormatsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>, // e.g., name, created_at, updated_at, creative_type
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "custom banner")]
    pub search: Option<String>, // Search by name
    #[schema(example = false)]
    pub is_public: Option<bool>, // Parameter to include public formats
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListCustomCreativeFormatsResponse {
    pub items: Vec<CustomCreativeFormat>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/formats/custom-creative-formats",
    tag = "Formats",
    security(("user_auth" = [])),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter formats by name"),
        ("is_public" = Option<bool>, Query, description = "Include public formats (default false)")
    ),
    responses(
        (status = 200, description = "List user's custom creative formats", body = ListCustomCreativeFormatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::get("")]
#[instrument(skip(pool, claims, params))]
pub async fn list_custom_creative_formats(
    pool: web::Data<PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    params: web::Query<ListCustomCreativeFormatsParams>,
) -> impl Responder {
    let user_id: Uuid = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;
    let param_is_public = params.is_public.unwrap_or(false);
    // Validate sort_by against allowed fields
    let sort_by = params
        .sort_by
        .clone()
        .filter(|s| ["name", "created_at", "updated_at"].contains(&s.as_str()))
        .unwrap_or_else(|| "created_at".into());
    let sort_order = params
        .sort_order
        .clone()
        .filter(|s| s == "asc" || s == "desc")
        .unwrap_or_else(|| "desc".into());

    // Total count query
    let total_count_result =
        count_formats(pool.get_ref(), user_id, param_is_public, params.search.clone()).await;

    let total_count = match total_count_result {
        Ok(count) => count,
        Err(e) => {
            log::error!(
                "Error counting custom creative formats (is_public: {param_is_public}): {e:?}"
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count custom creative formats".into(),
            });
        }
    };

    let items_result = list_formats(
        pool.get_ref(),
        user_id,
        param_is_public,
        params.search.clone(),
        &sort_by,
        &sort_order,
        limit,
        offset,
    )
    .await;

    match items_result {
        Ok(items) => HttpResponse::Ok().json(ListCustomCreativeFormatsResponse {
            items,
            total_count,
        }),
        Err(e) => {
            log::error!(
                "Database error fetching custom creative formats for user {user_id} (is_public: {param_is_public}): {e:?}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse { error: e.to_string() })
        }
    }
}
