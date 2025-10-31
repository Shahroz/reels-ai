//! Handler for retrieving a specific predefined collection by its ID.
//!
//! This endpoint allows users to fetch details of a single predefined collection.
//! The response includes the collection data along with any related information.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;

/// Enhanced predefined collection with additional metadata.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct PredefinedCollectionWithMetadata {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema_definition: serde_json::Value,
    pub ui_component_definition: serde_json::Value,
    #[schema(format = "date-time", value_type=String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type=String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(example = 5)]
    pub usage_count: i64,
    #[schema(example = "2024-01-15T10:30:00Z", value_type=String)]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Retrieves a specific predefined collection by ID.
///
/// Fetches the details of a single predefined collection if it exists.
/// The response includes usage statistics and metadata.
#[utoipa::path(
    get,
    path = "/api/predefined-collections/{collection_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the predefined collection to retrieve")
    ),
    responses(
        (status = 200, description = "Predefined collection details with metadata", body = PredefinedCollectionWithMetadata),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Predefined collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Predefined Collections"
)]
#[actix_web::get("/{collection_id}")]
pub async fn get_predefined_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    collection_id: actix_web::web::Path<uuid::Uuid>,
) -> actix_web::HttpResponse {
    let collection_id_to_fetch = collection_id.into_inner();

    // Query to get the predefined collection with usage statistics
    let result = sqlx::query!(
        r#"
        SELECT 
            pc.id,
            pc.name,
            pc.description,
            pc.schema_definition,
            pc.ui_component_definition,
            pc.created_at,
            pc.updated_at,
            COALESCE(COUNT(udc.id), 0)::bigint as usage_count,
            MAX(udc.created_at) as last_used_at
        FROM predefined_collections pc
        LEFT JOIN user_db_collections udc ON udc.source_predefined_collection_id = pc.id
        WHERE pc.id = $1
        GROUP BY pc.id, pc.name, pc.description, pc.schema_definition, pc.ui_component_definition, pc.created_at, pc.updated_at
        "#,
        collection_id_to_fetch
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let collection = PredefinedCollectionWithMetadata {
                id: row.id,
                name: row.name,
                description: row.description,
                schema_definition: row.schema_definition,
                ui_component_definition: row.ui_component_definition,
                created_at: row.created_at,
                updated_at: row.updated_at,
                usage_count: row.usage_count.unwrap_or(0),
                last_used_at: row.last_used_at,
            };
            log::info!("Retrieved predefined collection '{}'", collection.name);
            actix_web::HttpResponse::Ok().json(collection)
        }
        Ok(None) => {
            log::warn!("Predefined collection not found: {collection_id_to_fetch}");
            actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found.".into(),
            })
        }
        Err(e) => {
            log::error!("Failed to get predefined collection: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve predefined collection.".into(),
            })
        }
    }
} 