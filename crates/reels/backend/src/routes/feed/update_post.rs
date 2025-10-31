//! Update feed post endpoint

use actix_web::{put, web, HttpResponse, Responder};
use crate::routes::feed::types::{UpdateFeedPostRequest, FeedPostResponse};
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    put,
    path = "/api/feed/posts/{post_id}",
    tag = "Feed",
    request_body = UpdateFeedPostRequest,
    params(
        ("post_id" = String, Path, description = "Feed post UUID")
    ),
    responses(
        (status = 200, description = "Post updated successfully", body = FeedPostResponse),
        (status = 400, description = "Bad Request - Invalid input"),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 403, description = "Forbidden - Not post owner or assets not owned"),
        (status = 404, description = "Post not found or deleted"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[put("/posts/{post_id}")]
pub async fn update_feed_post(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    post_id: web::Path<String>,
    req: web::Json<UpdateFeedPostRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request = req.into_inner();
    
    // Parse post ID
    let post_id_uuid = match uuid::Uuid::parse_str(&post_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid post ID format: {}", e),
            });
        }
    };
    
    // Parse asset IDs if provided
    let asset_ids_option = if let Some(asset_id_strings) = request.asset_ids {
        let parsed: Result<Vec<uuid::Uuid>, _> = asset_id_strings
            .iter()
            .map(|s| uuid::Uuid::parse_str(s))
            .collect();
        
        match parsed {
            Ok(ids) => Some(ids),
            Err(e) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Invalid asset ID format: {}", e),
                });
            }
        }
    } else {
        None
    };
    
    // Call query function
    let args = crate::queries::feed::update_post::UpdateFeedPostArgs {
        post_id: post_id_uuid,
        user_id,
        caption: request.caption,
        asset_ids: asset_ids_option,
    };
    
    match crate::queries::feed::update_post::update_feed_post(&pool, args).await {
        Ok(true) => {
            log::info!("Updated feed post {} for user {}", post_id_uuid, user_id);
            
            // Fetch updated post to return
            match crate::queries::feed::get_post_by_id::get_feed_post_by_id(&pool, post_id_uuid).await {
                Ok(Some(post)) => {
                    HttpResponse::Ok().json(FeedPostResponse::from(post))
                }
                Ok(None) => {
                    HttpResponse::NotFound().json(ErrorResponse {
                        error: "Post not found after update".to_string(),
                    })
                }
                Err(e) => {
                    log::error!("Error fetching updated post: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Post updated but error fetching result".to_string(),
                    })
                }
            }
        }
        Ok(false) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Post not found, deleted, or you are not the owner".to_string(),
            })
        }
        Err(e) => {
            log::error!("Error updating feed post {}: {}", post_id_uuid, e);
            
            let error_msg = e.to_string();
            if error_msg.contains("Caption must be") || error_msg.contains("At least one asset") {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: error_msg,
                })
            } else if error_msg.contains("ownership validation failed") {
                HttpResponse::Forbidden().json(ErrorResponse {
                    error: "One or more assets are not owned by you".to_string(),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to update feed post".to_string(),
                })
            }
        }
    }
}

