//! Add prompt to favorites endpoint

use actix_web::{post, web, HttpResponse, Responder};
use crate::db::favorited_prompts::FavoritedPrompt;
use crate::routes::error_response::ErrorResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddFavoritePromptRequest {
    /// The enhancement prompt text to favorite
    #[schema(example = "make it brighter and more vibrant")]
    pub prompt_text: String,
    
    /// Optional title/label for organizing prompts
    #[schema(example = "Bright & Vibrant", nullable = true)]
    pub title: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/user-favorites/prompts",
    tag = "User Favorites",
    request_body = AddFavoritePromptRequest,
    responses(
        (status = 201, description = "Prompt added to favorites", body = FavoritedPrompt),
        (status = 400, description = "Bad Request - Empty prompt"),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[post("/prompts")]
pub async fn add_favorite_prompt(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<AddFavoritePromptRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    let request = req.into_inner();
    
    let args = crate::queries::favorited_prompts::add_favorite_prompt::AddFavoritePromptArgs {
        user_id,
        prompt_text: request.prompt_text,
        title: request.title,
    };
    
    match crate::queries::favorited_prompts::add_favorite_prompt::add_favorite_prompt(&pool, args).await {
        Ok(prompt_id) => {
            log::info!("User {} favorited prompt {}", user_id, prompt_id);
            
            // Fetch the favorited prompt to return full data
            match sqlx::query_as!(
                FavoritedPrompt,
                "SELECT id, user_id, prompt_text, title, created_at FROM favorited_prompts WHERE id = $1",
                prompt_id
            )
            .fetch_one(pool.as_ref())
            .await
            {
                Ok(prompt) => HttpResponse::Created().json(prompt),
                Err(e) => {
                    log::error!("Error fetching favorited prompt: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Prompt favorited but error fetching result".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            log::error!("Error adding favorite prompt for user {}: {}", user_id, error_msg);
            
            if error_msg.contains("cannot be empty") {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: error_msg,
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to add favorite prompt".to_string(),
                })
            }
        }
    }
}

