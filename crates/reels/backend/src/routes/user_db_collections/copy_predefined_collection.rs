//! Handler for copying a predefined collection.
//!
//! POST /api/user_db_collections/copy/{id} - Creates a new user collection from a predefined template by UUID.

use crate::auth::tokens::Claims;
use crate::db::user_db_collection::UserDbCollection;
use crate::routes::error_response::ErrorResponse;

use actix_web::{post, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct CopyPredefinedCollectionResponse {
    pub collection: UserDbCollection,
}

#[utoipa::path(
    post,
    path = "/api/user_db_collections/copy/{id}",
    params(
        ("id" = uuid::Uuid, Path, description = "UUID of the predefined collection to copy")
    ),
    tag = "User DB Collections",
    security(
      ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Collection copied successfully", body = CopyPredefinedCollectionResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Predefined collection not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[post("/copy/{id}")]
#[instrument(skip(pool, id, claims))]
pub async fn copy_predefined_collection(
    pool: web::Data<PgPool>,
    id: web::Path<uuid::Uuid>,
    claims: Claims,
) -> impl Responder {
    let user_id = claims.user_id;
    let collection_id = id.into_inner();

    // First, fetch the predefined collection to copy from
    let predefined_collection_result = sqlx::query!(
        r#"
        SELECT 
            id,
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            ui_component_definition AS "ui_component_definition: serde_json::Value"
        FROM predefined_collections
        WHERE id = $1
        "#,
        collection_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let predefined_collection = match predefined_collection_result {
        Ok(Some(collection)) => collection,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found".into(),
            });
        }
        Err(e) => {
            log::error!("Error fetching predefined collection: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch predefined collection".into(),
            });
        }
    };

    // Create the new user collection
    let new_collection_result = sqlx::query_as!(
        UserDbCollection,
        r#"
        INSERT INTO user_db_collections (
            user_id, 
            name, 
            description, 
            schema_definition, 
            source_predefined_collection_id,
            ui_component_definition
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id AS "id: uuid::Uuid",
            user_id AS "user_id: uuid::Uuid",
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            source_predefined_collection_id AS "source_predefined_collection_id: uuid::Uuid",
            ui_component_definition AS "ui_component_definition: serde_json::Value",
            created_at,
            updated_at
        "#,
        user_id,
        predefined_collection.name,
        predefined_collection.description,
        predefined_collection.schema_definition,
        predefined_collection.id,
        predefined_collection.ui_component_definition
    )
    .fetch_one(pool.get_ref())
    .await;

    match new_collection_result {
        Ok(collection) => {
            HttpResponse::Created().json(CopyPredefinedCollectionResponse { collection })
        }
        Err(e) => {
            log::error!("Error creating user collection from predefined: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create collection from template".into(),
            })
        }
    }
} 