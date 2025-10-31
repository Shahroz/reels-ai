//! Handler for toggling a user favorite (add if not exists, remove if exists).
use crate::auth::tokens::Claims;
use crate::db::favorites::{FavoriteEntityType, UserFavorite};
use crate::routes::error_response::ErrorResponse;
use crate::routes::user_favorites::create_favorite_request::CreateFavoriteRequest;
use actix_web::{post, web, HttpResponse};
use serde::{Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct ToggleFavoriteResponse {
    pub is_favorite: bool,
    pub favorite: Option<UserFavorite>,
}

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
    path = "/api/user-favorites/toggle",
    request_body = CreateFavoriteRequest,
    responses(
        (status = 200, description = "Favorite toggled", body = ToggleFavoriteResponse),
        (status = 400, description = "Invalid request payload or parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Entity not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "User Favorites",
    security(("user_auth" = []))
)]
#[post("/toggle")]
#[instrument(skip(pool, claims, req))]
pub async fn toggle_favorite(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateFavoriteRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let authenticated_user_id = claims.user_id;
    let request_data = req.into_inner();

    log::info!("Received toggle_favorite request. User ID: {}, Entity ID: {}, Entity Type: {}",
               authenticated_user_id, request_data.entity_id, request_data.entity_type);

    match request_data.entity_type.as_str() {
        "style" | "creative" | "document" => (),
        _ => return Err(actix_web::error::ErrorBadRequest(format!("Unsupported entity_type: {}", request_data.entity_type))),
    }

    // Check if entity exists
    match can_user_favorite_entity(&pool, authenticated_user_id, request_data.entity_id, &request_data.entity_type).await {
        Ok(true) => (),
        Ok(false) => {
            log::warn!("Entity {} of type {} not found for user {}", request_data.entity_id, request_data.entity_type, authenticated_user_id);
            return Err(actix_web::error::ErrorNotFound("Entity not found."));
        }
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("Failed to check entity existence")),
    }

    let entity_type_enum = match FavoriteEntityType::from_str(&request_data.entity_type.to_lowercase()) {
        Ok(et) => et,
        Err(_) => return Err(actix_web::error::ErrorBadRequest(format!("Invalid entity_type: {}", request_data.entity_type))),
    };

    // Check if favorite already exists
    let existing_favorite = sqlx::query_as!(
        UserFavorite,
        r#"
        SELECT id, user_id, entity_id, entity_type as "entity_type: FavoriteEntityType", created_at, updated_at
        FROM user_favorites
        WHERE user_id = $1 AND entity_id = $2 AND entity_type = $3
        "#,
        authenticated_user_id,
        request_data.entity_id,
        entity_type_enum as FavoriteEntityType
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| {
        log::error!("DB error checking existing favorite: {e}");
        actix_web::error::ErrorInternalServerError("Failed to check existing favorite")
    })?;

    if let Some(favorite) = existing_favorite {
        // Favorite exists, remove it
        let deleted_count = sqlx::query!(
            "DELETE FROM user_favorites WHERE id = $1 AND user_id = $2",
            favorite.id,
            authenticated_user_id
        )
        .execute(&**pool)
        .await
        .map_err(|e| {
            log::error!("DB error deleting favorite: {e}");
            actix_web::error::ErrorInternalServerError("Failed to remove favorite")
        })?
        .rows_affected();

        if deleted_count == 0 {
            log::warn!("No favorite deleted for toggle operation");
            return Err(actix_web::error::ErrorInternalServerError("Failed to remove favorite"));
        }

        log::info!("Successfully removed favorite {} for user {} on entity {} ({})",
                   favorite.id, authenticated_user_id, request_data.entity_id, request_data.entity_type);

        let response = ToggleFavoriteResponse {
            is_favorite: false,
            favorite: None,
        };

        Ok(HttpResponse::Ok().json(response))
    } else {
        // Favorite doesn't exist, create it
        let new_favorite = sqlx::query_as!(
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
        .await
        .map_err(|e| {
            log::error!("DB error creating favorite: {e}");
            actix_web::error::ErrorInternalServerError("Failed to create favorite")
        })?;

        log::info!("Successfully created favorite {} for user {} on entity {} ({})",
                   new_favorite.id, authenticated_user_id, request_data.entity_id, request_data.entity_type);

        let response = ToggleFavoriteResponse {
            is_favorite: true,
            favorite: Some(new_favorite),
        };

        Ok(HttpResponse::Ok().json(response))
    }
}