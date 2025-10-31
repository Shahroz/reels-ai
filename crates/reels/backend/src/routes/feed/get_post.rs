//! Get single feed post endpoint

use actix_web::{get, web, HttpResponse, Responder};
use crate::routes::feed::types::FeedPostResponse;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    get,
    path = "/api/feed/posts/{post_id}",
    tag = "Feed",
    params(
        ("post_id" = String, Path, description = "Feed post UUID")
    ),
    responses(
        (status = 200, description = "Post retrieved successfully", body = FeedPostResponse),
        (status = 400, description = "Bad Request - Invalid UUID format"),
        (status = 404, description = "Post not found or deleted"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/posts/{post_id}")]
pub async fn get_feed_post(
    pool: web::Data<sqlx::PgPool>,
    post_id: web::Path<String>,
) -> impl Responder {
    let post_id_uuid = match uuid::Uuid::parse_str(&post_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid post ID format: {}", e),
            });
        }
    };
    
    match crate::queries::feed::get_post_by_id::get_feed_post_by_id(&pool, post_id_uuid).await {
        Ok(Some(post)) => {
            HttpResponse::Ok().json(FeedPostResponse::from(post))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Post not found or has been deleted".to_string(),
            })
        }
        Err(e) => {
            log::error!("Error fetching post {}: {}", post_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch post".to_string(),
            })
        }
    }
}

