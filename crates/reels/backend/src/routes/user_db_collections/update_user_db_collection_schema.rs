//! Handler for updating the schema definition of an existing user DB collection.
//!
//! This endpoint allows an authenticated user to update the JSON schema definition
//! of a custom database collection they own.
//! Adheres to 'one item per file' and FQN guidelines.

use crate::routes::user_db_collections::update_user_db_collection_schema_request::UpdateUserDbCollectionSchemaRequest;

/// Updates an existing DB collection's schema definition.
///
/// Allows modification of the `schema_definition` of a collection
/// owned by the authenticated user, either directly or via LLM instruction.
/// The `updated_at` field is also updated.
#[utoipa::path(
    put,
    path = "/api/user-db-collections/{collection_id}/schema",
    request_body = UpdateUserDbCollectionSchemaRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the collection whose schema is to be updated")
    ),
    responses(
        (status = 200, description = "User DB collection schema updated successfully", body = crate::db::user_db_collection::UserDbCollection),
        (status = 400, description = "Invalid request payload or invalid JSON schema", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Collection not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::put("/{collection_id}/schema")]
pub async fn update_user_db_collection_schema(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id: actix_web::web::Path<uuid::Uuid>,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::update_user_db_collection_schema_request::UpdateUserDbCollectionSchemaRequest>,
) -> actix_web::HttpResponse {
    match crate::queries::user_db_collections::update_user_db_collection_schema_query::update_user_db_collection_schema_query(
        pool.get_ref(),
        claims.user_id,
        collection_id.into_inner(),
        req_body.into_inner().payload,
    )
    .await
    {
        std::result::Result::Ok(collection) => actix_web::HttpResponse::Ok().json(collection),
        std::result::Result::Err(e) => {
            log::error!("Error in update_user_db_collection_schema route: {e:?}");
            match e {
                crate::queries::user_db_collections::update_user_db_collection_schema_query::UpdateSchemaError::NotFound => {
                    actix_web::HttpResponse::NotFound().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::string::String::from("Collection not found or not owned by user."),
                        },
                    )
                }
                crate::queries::user_db_collections::update_user_db_collection_schema_query::UpdateSchemaError::LlmError(msg) => {
                    actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::format!("LLM failed to process schema update instruction: {msg}"),
                        },
                    )
                }
                crate::queries::user_db_collections::update_user_db_collection_schema_query::UpdateSchemaError::FetchCurrentSchemaError(db_err) => {
                    actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::format!("Failed to retrieve current schema: {db_err}"),
                        },
                    )
                }
                crate::queries::user_db_collections::update_user_db_collection_schema_query::UpdateSchemaError::DatabaseUpdateError(db_err) => {
                    actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::format!("Failed to save updated collection schema: {db_err}"),
                        },
                    )
                }
                crate::queries::user_db_collections::update_user_db_collection_schema_query::UpdateSchemaError::JsonError(json_err) => {
                    actix_web::HttpResponse::BadRequest().json( // Or InternalServerError depending on context
                        crate::routes::error_response::ErrorResponse {
                            error: std::format!("JSON processing error: {json_err}"),
                        },
                    )
                }
            }
        }
    }
}
