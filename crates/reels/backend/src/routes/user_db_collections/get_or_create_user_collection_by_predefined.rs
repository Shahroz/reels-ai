//! Handler for fetching or creating a user collection by predefined collection ID.
//!
//! This endpoint allows users to get their collection based on a predefined collection ID.
//! If the user doesn't have a collection for that predefined collection, it creates one.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;
use crate::routes::user_db_collections::user_collection_service;

/// Response structure for the get or create user collection endpoint.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct GetOrCreateUserCollectionResponse {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    #[schema(format = "uuid", value_type=String, nullable = true)]
    pub source_predefined_collection_id: Option<uuid::Uuid>,
    pub ui_component_definition: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Fetches or creates a user collection based on a predefined collection ID.
///
/// This endpoint first checks if the user already has a collection for the given
/// predefined collection. If not, it creates a new user collection based on the
/// predefined collection template.
#[utoipa::path(
    get,
    path = "/api/user-db-collections/by-predefined/{predefined_collection_id}",
    params(
        ("predefined_collection_id" = uuid::Uuid, Path, description = "ID of the predefined collection")
    ),
    responses(
        (status = 200, description = "User collection details (existing or newly created)", body = GetOrCreateUserCollectionResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Predefined collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::get("/by-predefined/{predefined_collection_id}")]
pub async fn get_or_create_user_collection_by_predefined(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    predefined_collection_id: actix_web::web::Path<uuid::Uuid>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;
    let predefined_id = predefined_collection_id.into_inner();

    // Fetch predefined collection by ID
    let predefined_collection = match user_collection_service::fetch_predefined_collection_by_id(
        pool.get_ref(),
        predefined_id,
    ).await {
        Ok(data) => data,
        Err(response) => return response,
   };

   // Check if user already has a collection for this predefined collection
   let _existing_collection: Option<crate::db::user_db_collection::UserDbCollection> = match user_collection_service::check_existing_user_collection(
       pool.get_ref(),
       user_id,
       predefined_collection.id,
    ).await {
        Ok(Some(collection)) => {
            log::info!("Found existing user collection for predefined collection: {predefined_id}");
            let response = GetOrCreateUserCollectionResponse {
                id: collection.id,
                user_id: collection.user_id,
                name: collection.name,
                description: collection.description,
                schema_definition: collection.schema_definition,
                source_predefined_collection_id: collection.source_predefined_collection_id,
                ui_component_definition: collection.ui_component_definition,
                created_at: collection.created_at,
                updated_at: collection.updated_at,
            };
            return actix_web::HttpResponse::Ok().json(response);
        }
        Ok(None) => None,
        Err(response) => return response,
    };

    // Create new user collection based on predefined collection
    let new_collection = match user_collection_service::create_user_collection_from_predefined(
        pool.get_ref(),
        user_id,
        &predefined_collection,
    ).await {
        Ok(collection) => collection,
        Err(response) => return response,
    };

    log::info!("Created new user collection for predefined collection: {predefined_id}");
    let response = GetOrCreateUserCollectionResponse {
        id: new_collection.id,
        user_id: new_collection.user_id,
        name: new_collection.name,
        description: new_collection.description,
        schema_definition: new_collection.schema_definition,
        source_predefined_collection_id: new_collection.source_predefined_collection_id,
        ui_component_definition: new_collection.ui_component_definition,
        created_at: new_collection.created_at,
        updated_at: new_collection.updated_at,
    };
    actix_web::HttpResponse::Ok().json(response)
} 
