//! Create feed post endpoint

use actix_web::{post, web, HttpResponse, Responder};
use crate::routes::feed::types::{CreateFeedPostRequest, FeedPostResponse};
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    post,
    path = "/api/feed/posts",
    tag = "Feed",
    request_body = CreateFeedPostRequest,
    responses(
        (status = 201, description = "Post created successfully", body = FeedPostResponse),
        (status = 400, description = "Bad Request - Invalid input (caption length, no assets, invalid UUIDs, etc.)"),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 403, description = "Forbidden - Assets not owned by user"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[post("/posts")]
pub async fn create_feed_post(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<CreateFeedPostRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request = req.into_inner();
    
    // Parse asset IDs from strings to UUIDs
    let asset_ids: Result<Vec<uuid::Uuid>, _> = request
        .asset_ids
        .iter()
        .map(|s| uuid::Uuid::parse_str(s))
        .collect();
    
    let asset_ids = match asset_ids {
        Ok(ids) => ids,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid asset ID format: {}", e),
            });
        }
    };
    
    // Call query function
    let args = crate::queries::feed::create_post::CreateFeedPostArgs {
        user_id,
        caption: request.caption,
        asset_ids: asset_ids.clone(),
    };
    
    match crate::queries::feed::create_post::create_feed_post(&pool, args).await {
        Ok(result) => {
            log::info!("Created feed post {} with {} assets for user {}", 
                result.post_id, result.assets_added, user_id);
            
            // Fetch the created post to return full response
            match crate::queries::feed::get_post_by_id::get_feed_post_by_id(&pool, result.post_id).await {
                Ok(Some(post)) => {
                    HttpResponse::Created().json(FeedPostResponse::from(post))
                }
                Ok(None) => {
                    // Should not happen, but handle gracefully
                    log::error!("Created post {} but cannot fetch it back", result.post_id);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Post created but cannot be retrieved".to_string(),
                    })
                }
                Err(e) => {
                    log::error!("Error fetching created post: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Post created but error fetching: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            log::error!("Error creating feed post for user {}: {}", user_id, e);
            
            // Determine appropriate status code based on error message
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
                    error: "Failed to create feed post".to_string(),
                })
            }
        }
    }
}

