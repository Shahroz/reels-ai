//! Handler for listing all logo collections.
//!
//! GET /api/logo-collections - Returns logo collections belonging to the authenticated user.

use crate::schemas::logo_collection_schemas::LogoCollectionSummaryResponse;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Deserialize, Debug, ToSchema, utoipa::IntoParams)]
pub struct ListLogoCollectionsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "brand")]
    pub search: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListLogoCollectionsResponse {
    pub items: Vec<LogoCollectionSummaryResponse>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/logo-collections",
    tag = "Logo Collections",
    params(ListLogoCollectionsParams),
    responses(
        (status = 200, description = "Success", body = ListLogoCollectionsResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[get("")]
#[instrument(skip(pool, claims))]
pub async fn list_logo_collections(
    pool: web::Data<PgPool>,
    query: web::Query<ListLogoCollectionsParams>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let user_id = claims.user_id;

    // For now, implement basic listing without pagination/search
    // TODO: Add pagination, sorting, and search functionality
    let result = crate::queries::logo_collections::list_logo_collections::list_logo_collections(
        pool.get_ref(),
        user_id,
    )
    .await;

    match result {
        Ok(collections) => {
            let total_count = collections.len() as i64;
            HttpResponse::Ok().json(ListLogoCollectionsResponse {
                items: collections,
                total_count,
            })
        }
        Err(err) => {
            log::error!("Database error listing logo collections for user {}: {:?}", user_id, err);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to list logo collections".to_string(),
            })
        }
    }
}
