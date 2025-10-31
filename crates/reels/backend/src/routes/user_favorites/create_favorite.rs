//! Handler for creating a user favorite.
use crate::auth::tokens::Claims;
use crate::db::favorites::{FavoriteEntityType, UserFavorite};
use crate::routes::error_response::ErrorResponse;
use crate::routes::user_favorites::create_favorite_request::CreateFavoriteRequest;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use std::str::FromStr;

// Helper to check if entity exists and user has access
async fn can_user_favorite_entity(
    pool: &PgPool,
    _user_id: Uuid,
    entity_id: Uuid,
    entity_type: &str,
) -> Result<bool, HttpResponse> {
    let entity_exists = match entity_type {
        "style" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1)", entity_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking style existence: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check entity existence"))
                })?.unwrap_or(false)
        }
        "creative" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM creatives WHERE id = $1)", entity_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking creative existence: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check entity existence"))
                })?.unwrap_or(false)
        }
        "document" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)", entity_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking document existence: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check entity existence"))
                })?.unwrap_or(false)
        }
        _ => return Err(HttpResponse::BadRequest().json(ErrorResponse::from("Unsupported entity type for favoriting."))),
    };

    Ok(entity_exists)
}

#[utoipa::path(
    post,
    path = "/api/user-favorites",
    request_body = CreateFavoriteRequest,
    responses(
        (status = 201, description = "Favorite created", body = UserFavorite),
        (status = 400, description = "Invalid request payload or parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Entity not found", body = ErrorResponse),
        (status = 409, description = "Favorite already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "User Favorites",
    security(("user_auth" = []))
)]
#[post("")]
#[instrument(skip(pool, claims, req))]
pub async fn create_favorite(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateFavoriteRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let request_data = req.into_inner();

    log::info!("Received create_favorite request. User ID: {}, Entity ID: {}, Entity Type: {}",
               authenticated_user_id, request_data.entity_id, request_data.entity_type);

    match request_data.entity_type.as_str() {
        "style" | "creative" | "document" => (),
        _ => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Unsupported entity_type: {}", request_data.entity_type))),
    }

    // Check if entity exists
    match can_user_favorite_entity(&pool, authenticated_user_id, request_data.entity_id, &request_data.entity_type).await {
        Ok(true) => (),
        Ok(false) => {
            log::warn!("Entity {} of type {} not found for user {}", request_data.entity_id, request_data.entity_type, authenticated_user_id);
            return HttpResponse::NotFound().json(ErrorResponse::from("Entity not found."));
        }
        Err(resp) => return resp,
    }

    let entity_type_enum = match FavoriteEntityType::from_str(&request_data.entity_type.to_lowercase()) {
        Ok(et) => et,
        Err(_) => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Invalid entity_type: {}", request_data.entity_type))),
    };

    // Check if favorite already exists
    let existing_favorite = match sqlx::query_scalar!(
        "SELECT id FROM user_favorites WHERE user_id = $1 AND entity_id = $2 AND entity_type = $3",
        authenticated_user_id,
        request_data.entity_id,
        entity_type_enum as FavoriteEntityType
    )
    .fetch_optional(&**pool)
    .await {
        Ok(favorite) => favorite,
        Err(e) => {
            log::error!("DB error checking existing favorite: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check existing favorite"));
        }
    };

    if existing_favorite.is_some() {
        return HttpResponse::Conflict().json(ErrorResponse::from("Favorite already exists"));
    }

    // Create the favorite
    let favorite = match sqlx::query_as!(
        UserFavorite,
        r#"
        INSERT INTO user_favorites (user_id, entity_id, entity_type)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, entity_id, entity_type as "entity_type: FavoriteEntityType", created_at, updated_at
        "#,
        authenticated_user_id,
        request_data.entity_id,
        entity_type_enum as FavoriteEntityType
    )
    .fetch_one(&**pool)
    .await {
        Ok(favorite) => favorite,
        Err(e) => {
            log::error!("DB error creating favorite: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to create favorite"));
        }
    };

    log::info!("Successfully created favorite {} for user {} on entity {} ({})",
               favorite.id, authenticated_user_id, request_data.entity_id, request_data.entity_type);

    HttpResponse::Created().json(favorite)
}