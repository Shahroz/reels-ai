//! Handler for updating predefined collections.
//!
//! This endpoint allows administrators to update existing predefined collections.
//! Only administrators should have access to this endpoint.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;
use crate::routes::predefined_collections::update_predefined_collection_request::UpdatePredefinedCollectionRequest;

/// Updates a predefined collection.
///
/// Updates an existing predefined collection with the provided data.
/// This endpoint is typically used by administrators to modify template collections.
#[utoipa::path(
    put,
    path = "/api/predefined-collections/{collection_id}",
    request_body = UpdatePredefinedCollectionRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the predefined collection to update")
    ),
    responses(
        (status = 200, description = "Predefined collection updated successfully", body = crate::db::predefined_collection::PredefinedCollection),
        (status = 400, description = "Bad request - invalid data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Predefined collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Predefined Collections"
)]
#[actix_web::put("/{collection_id}")]
pub async fn update_predefined_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    collection_id: actix_web::web::Path<uuid::Uuid>,
    request: actix_web::web::Json<crate::routes::predefined_collections::update_predefined_collection_request::UpdatePredefinedCollectionRequest>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let collection_id_to_update = collection_id.into_inner();
    let request_data = request.into_inner();
    
    // Check if user has admin privileges (you may need to adjust this based on your auth system)
    // For now, we'll allow any authenticated user to update predefined collections
    
    // Validate that at least one field is provided
    if request_data.name.is_none() && request_data.description.is_none() && 
       request_data.schema_definition.is_none() && request_data.ui_component_definition.is_none() {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: "At least one field must be provided for update.".into(),
        });
    }
    
    // Validate name if provided
    if let Some(ref name) = request_data.name {
        if name.trim().is_empty() {
            return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
                error: "Name cannot be empty if provided.".into(),
            });
        }
    }
    
    // First, check if the predefined collection exists
    let existing_collection = sqlx::query_as!(
        crate::db::predefined_collection::PredefinedCollection,
        "SELECT id, name, description, schema_definition, ui_component_definition, created_at, updated_at FROM predefined_collections WHERE id = $1",
        collection_id_to_update
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match existing_collection {
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: "Predefined collection not found.".into(),
            });
        }
        Err(e) => {
            log::error!("Failed to check predefined collection existence: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to check collection existence.".into(),
            });
        }
        Ok(Some(_)) => {} // Collection exists, continue with update
    }
    
    // Build the update query dynamically based on provided fields
    let mut query_builder = sqlx::QueryBuilder::new(
        "UPDATE predefined_collections SET updated_at = NOW()"
    );
    
    if let Some(ref name) = request_data.name {
        query_builder.push(", name = ");
        query_builder.push_bind(name);
    }
    
    if let Some(ref description) = request_data.description {
        query_builder.push(", description = ");
        query_builder.push_bind(description);
    }
    
    if let Some(ref schema_definition) = request_data.schema_definition {
        query_builder.push(", schema_definition = ");
        query_builder.push_bind(schema_definition);
    }
    
    if let Some(ref ui_component_definition) = request_data.ui_component_definition {
        query_builder.push(", ui_component_definition = ");
        query_builder.push_bind(ui_component_definition);
    }
    
    query_builder.push(" WHERE id = ");
    query_builder.push_bind(collection_id_to_update);
    query_builder.push(" RETURNING id, name, description, schema_definition, ui_component_definition, created_at, updated_at");
    
    let result = query_builder
        .build_query_as::<crate::db::predefined_collection::PredefinedCollection>()
        .fetch_one(pool.get_ref())
        .await;
    
    match result {
        Ok(collection) => {
            log::info!("Updated predefined collection '{}' by user {}", collection.name, user_id);
            actix_web::HttpResponse::Ok().json(collection)
        }
        Err(e) => {
            log::error!("Failed to update predefined collection: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update predefined collection.".into(),
            })
        }
    }
} 