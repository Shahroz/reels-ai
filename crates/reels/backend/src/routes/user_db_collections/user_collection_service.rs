//! Shared service functions for user collection operations.
//!
//! This module contains shared logic for user collection operations to avoid code duplication.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;
use crate::db::user_db_collection::UserDbCollection;

/// Result type for user collection operations
pub type UserCollectionResult = Result<UserDbCollection, actix_web::HttpResponse>;

/// Fetches a predefined collection by ID
pub async fn fetch_predefined_collection_by_id(
    pool: &sqlx::PgPool,
    predefined_id: uuid::Uuid,
) -> Result<PredefinedCollectionData, actix_web::HttpResponse> {
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
        predefined_id
    )
    .fetch_optional(pool)
    .await;

    match predefined_collection_result {
        Ok(Some(collection)) => Ok(PredefinedCollectionData {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            schema_definition: collection.schema_definition,
            ui_component_definition: collection.ui_component_definition,
        }),
        Ok(None) => {
            log::warn!("Predefined collection not found: {predefined_id}");
            Err(actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found.".into(),
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch predefined collection: {e:?}");
            Err(actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch predefined collection.".into(),
            }))
        }
    }
}

/// Fetches a predefined collection by name
pub async fn fetch_predefined_collection_by_name(
    pool: &sqlx::PgPool,
    predefined_name: &str,
) -> Result<PredefinedCollectionData, actix_web::HttpResponse> {
    let predefined_collection_result = sqlx::query!(
        r#"
        SELECT 
            id,
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            ui_component_definition AS "ui_component_definition: serde_json::Value"
        FROM predefined_collections
        WHERE name = $1
        "#,
        predefined_name
    )
    .fetch_optional(pool)
    .await;

    match predefined_collection_result {
        Ok(Some(collection)) => Ok(PredefinedCollectionData {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            schema_definition: collection.schema_definition,
            ui_component_definition: collection.ui_component_definition,
        }),
        Ok(None) => {
            log::warn!("Predefined collection not found: {predefined_name}");
            Err(actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found.".into(),
            }))
        }
        Err(e) => {
            log::error!("Failed to fetch predefined collection: {e:?}");
            Err(actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch predefined collection.".into(),
            }))
        }
    }
}

/// Checks if user already has a collection for the given predefined collection
pub async fn check_existing_user_collection(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    predefined_collection_id: uuid::Uuid,
) -> Result<Option<UserDbCollection>, actix_web::HttpResponse> {
    let existing_collection_result = sqlx::query_as!(
        UserDbCollection,
        r#"
        SELECT 
            id AS "id: uuid::Uuid",
            user_id AS "user_id: uuid::Uuid",
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            source_predefined_collection_id AS "source_predefined_collection_id: uuid::Uuid",
            ui_component_definition AS "ui_component_definition: serde_json::Value",
            created_at,
            updated_at
        FROM user_db_collections
        WHERE user_id = $1 AND source_predefined_collection_id = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user_id,
        predefined_collection_id
    )
    .fetch_optional(pool)
    .await;

    match existing_collection_result {
        Ok(existing_collection) => Ok(existing_collection),
        Err(e) => {
            log::error!("Failed to check existing user collection: {e:?}");
            Err(actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to check existing user collection.".into(),
            }))
        }
    }
}

/// Creates a new user collection based on predefined collection data
pub async fn create_user_collection_from_predefined(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    predefined_data: &PredefinedCollectionData,
) -> Result<UserDbCollection, actix_web::HttpResponse> {
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
        predefined_data.name,
        predefined_data.description,
        predefined_data.schema_definition,
        predefined_data.id,
        predefined_data.ui_component_definition
    )
    .fetch_one(pool)
    .await;

    match new_collection_result {
        Ok(new_collection) => {
            log::info!("Created new user collection for predefined collection: {}", predefined_data.name);
            Ok(new_collection)
        }
        Err(e) => {
            log::error!("Failed to create user collection: {e:?}");
            Err(actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create user collection.".into(),
            }))
        }
    }
}

/// Data structure for predefined collection information
#[derive(Debug, Clone)]
pub struct PredefinedCollectionData {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    pub ui_component_definition: serde_json::Value,
} 