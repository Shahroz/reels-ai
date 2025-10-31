//! Handler for creating predefined collections.
//!
//! This endpoint allows administrators to create new predefined collections
//! that can be used as templates by users.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::error_response::ErrorResponse;
use crate::routes::predefined_collections::create_predefined_collection_request::CreatePredefinedCollectionRequest;

/// Creates a new predefined collection.
///
/// Creates a new predefined collection with the provided schema and UI component definitions.
/// This endpoint is typically used by administrators to create template collections.
#[utoipa::path(
    post,
    path = "/api/predefined-collections",
    request_body = CreatePredefinedCollectionRequest,
    responses(
        (status = 201, description = "Predefined collection created successfully", body = crate::db::predefined_collection::PredefinedCollection),
        (status = 400, description = "Bad request - invalid data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Predefined Collections"
)]
#[actix_web::post("")]
pub async fn create_predefined_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    request: actix_web::web::Json<crate::routes::predefined_collections::create_predefined_collection_request::CreatePredefinedCollectionRequest>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    
    // Check if user has admin privileges (you may need to adjust this based on your auth system)
    // For now, we'll allow any authenticated user to create predefined collections
    
    let request_data = request.into_inner();
    
    // Validate required fields
    if request_data.name.trim().is_empty() {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: "Name is required and cannot be empty.".into(),
        });
    }
    
    // Insert the predefined collection
    let result = sqlx::query_as!(
        crate::db::predefined_collection::PredefinedCollection,
        r#"
        INSERT INTO predefined_collections (name, description, schema_definition, ui_component_definition)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, description, schema_definition, ui_component_definition, created_at, updated_at
        "#,
        request_data.name,
        request_data.description,
        request_data.schema_definition,
        request_data.ui_component_definition
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match result {
        Ok(collection) => {
            log::info!("Created predefined collection '{}' by user {}", collection.name, user_id);
            actix_web::HttpResponse::Created().json(collection)
        }
        Err(e) => {
            log::error!("Failed to create predefined collection: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create predefined collection.".into(),
            })
        }
    }
} 