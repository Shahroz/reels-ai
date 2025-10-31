//! Delete feed post endpoint (soft delete)

use actix_web::{delete, web, HttpResponse, Responder};
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    delete,
    path = "/api/feed/posts/{post_id}",
    tag = "Feed",
    params(
        ("post_id" = String, Path, description = "Feed post UUID")
    ),
    responses(
        (status = 204, description = "Post deleted successfully"),
        (status = 400, description = "Bad Request - Invalid UUID format"),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 403, description = "Forbidden - Not post owner"),
        (status = 404, description = "Post not found or already deleted"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[delete("/posts/{post_id}")]
pub async fn delete_feed_post(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    post_id: web::Path<String>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    // Parse post ID
    let post_id_uuid = match uuid::Uuid::parse_str(&post_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid post ID format: {}", e),
            });
        }
    };
    
    match crate::queries::feed::delete_post::delete_feed_post(&pool, post_id_uuid, user_id).await {
        Ok(true) => {
            log::info!("Deleted feed post {} by user {}", post_id_uuid, user_id);
            HttpResponse::NoContent().finish()
        }
        Ok(false) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Post not found, already deleted, or you are not the owner".to_string(),
            })
        }
        Err(e) => {
            log::error!("Error deleting feed post {}: {}", post_id_uuid, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete feed post".to_string(),
            })
        }
    }
}

