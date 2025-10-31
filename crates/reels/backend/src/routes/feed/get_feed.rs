//! Get feed list endpoint with pagination

use actix_web::{get, web, HttpResponse, Responder};
use crate::routes::feed::types::{GetFeedResponse, FeedPostResponse};
use crate::routes::error_response::ErrorResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GetFeedQueryParams {
    /// Page number (1-indexed, defaults to 1)
    #[serde(default = "default_page")]
    pub page: i64,
    
    /// Number of posts per page (defaults to 20, max 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 { 1 }
fn default_limit() -> i64 { 20 }

#[utoipa::path(
    get,
    path = "/api/feed/posts",
    tag = "Feed",
    params(GetFeedQueryParams),
    responses(
        (status = 200, description = "Feed retrieved successfully", body = GetFeedResponse),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/posts")]
pub async fn get_feed(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<GetFeedQueryParams>,
) -> impl Responder {
    let params = crate::queries::feed::get_feed::GetFeedParams {
        page: query.page,
        limit: query.limit,
    };
    
    match crate::queries::feed::get_feed::get_feed(&pool, params).await {
        Ok(result) => {
            log::info!("Fetched feed page {} with {} posts", result.page, result.posts.len());
            
            let response = GetFeedResponse {
                posts: result.posts.into_iter().map(FeedPostResponse::from).collect(),
                page: result.page,
                total_pages: result.total_pages,
                total_count: result.total_count,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Error fetching feed: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch feed".to_string(),
            })
        }
    }
}

