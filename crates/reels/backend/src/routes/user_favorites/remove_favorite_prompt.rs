//! Remove prompt from favorites endpoint

use actix_web::{delete, web, HttpResponse, Responder};
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    delete,
    path = "/api/user-favorites/prompts/{prompt_id}",
    tag = "User Favorites",
    params(
        ("prompt_id" = String, Path, description = "Favorited prompt UUID")
    ),
    responses(
        (status = 204, description = "Prompt removed from favorites"),
        (status = 400, description = "Bad Request - Invalid UUID"),
        (status = 401, description = "Unauthorized - Authentication required"),
        (status = 404, description = "Prompt not found or not owned"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[delete("/prompts/{prompt_id}")]
pub async fn remove_favorite_prompt(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    prompt_id: web::Path<String>,
) -> impl Responder {
    let user_id = claims.user_id;
    
    let prompt_id_uuid = match uuid::Uuid::parse_str(&prompt_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid prompt ID format: {}", e),
            });
        }
    };
    
    match crate::queries::favorited_prompts::remove_favorite_prompt::remove_favorite_prompt(&pool, prompt_id_uuid, user_id).await {
        Ok(true) => {
            log::info!("User {} removed favorite prompt {}", user_id, prompt_id_uuid);
            HttpResponse::NoContent().finish()
        }
        Ok(false) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Favorite prompt not found or not owned by you".to_string(),
            })
        }
        Err(e) => {
            log::error!("Error removing favorite prompt {}: {}", prompt_id_uuid, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to remove favorite prompt".to_string(),
            })
        }
    }
}

