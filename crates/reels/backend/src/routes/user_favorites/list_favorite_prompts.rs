//! List favorite prompts endpoint

use actix_web::{get, web, HttpResponse, Responder};
use crate::routes::error_response::ErrorResponse;
use crate::db::favorited_prompts::FavoritedPrompt;

#[utoipa::path(
    get,
    path = "/api/user-favorites/prompts",
    tag = "User Favorites",
    responses(
        (status = 200, description = "List of favorite prompts", body = Vec<FavoritedPrompt>),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[get("/prompts")]
pub async fn list_favorite_prompts(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    match crate::queries::favorited_prompts::list_favorite_prompts::list_favorite_prompts(&pool, user_id).await {
        Ok(prompts) => {
            log::info!("User {} listed {} favorite prompts", user_id, prompts.len());
            HttpResponse::Ok().json(prompts)
        }
        Err(e) => {
            log::error!("Error listing favorite prompts for user {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to list favorite prompts".to_string(),
            })
        }
    }
}

