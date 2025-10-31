//! Handler for creating a new user DB collection.
//!
//! This endpoint allows an authenticated user to create a new custom database collection.
//! It expects the collection's name, an optional description, and its schema definition.
//! The new collection is associated with the authenticated user.
//! Adheres to 'one item per file' and FQN guidelines.

/// Creates a new user DB collection.
///
///
/// Authenticated users can define new collections by providing a name,
/// an optional description, and a JSON schema for the items within the collection.
#[utoipa::path(
    post,
    path = "/api/user-db-collections",
    request_body = crate::routes::user_db_collections::create_user_db_collection_request::CreateUserDbCollectionRequest,
    responses(
        (status = 201, description = "User DB collection created successfully", body = crate::db::user_db_collection::UserDbCollection),
        (status = 400, description = "Invalid request payload", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::post("")]
#[allow(clippy::too_many_lines)] // Allowed for now, consider refactoring if grows further
pub async fn create_user_db_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::create_user_db_collection_request::CreateUserDbCollectionRequest>,
) -> actix_web::HttpResponse {
    let user_id = claims.user_id;

    match crate::queries::user_db_collections::create_user_db_collection_query::create_user_db_collection_query(
        pool.get_ref(),
        user_id,
        req_body.name.clone(),
        req_body.description.clone(),
        req_body.schema_definition.clone(),
    )
    .await
    {
        Ok(collection) => actix_web::HttpResponse::Created().json(collection),
        Err(e) => {
            // Log the detailed error from the query function
            log::error!("Failed to create user DB collection via query: {e:?}");

            // Check if the error message indicates a client-side issue (e.g., bad schema format)
            // This is a simple check; more robust error typing from the query would be better.
            if e.to_string().contains("Invalid initial schema definition") {
                return actix_web::HttpResponse::BadRequest().json(
                    crate::routes::error_response::ErrorResponse {
                        error: format!("Invalid request: {e}"),
                    },
                );
            }

            // For other errors (LLM, DB), return InternalServerError
            actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: format!("Internal server error: {e}"),
                },
            )
        }
    }
}