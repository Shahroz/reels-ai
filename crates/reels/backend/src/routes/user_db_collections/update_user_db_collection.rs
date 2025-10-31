//! Handler for updating an existing user DB collection.
//!
//! This endpoint allows an authenticated user to update the metadata (name, description)
//! of a custom database collection they own. Schema definition cannot be updated here.
//! Adheres to 'one item per file' and FQN guidelines.

/// Updates an existing DB collection's metadata.
///
/// Allows modification of the name and/or description of a collection
/// owned by the authenticated user.
#[utoipa::path(
    put,
    path = "/api/user-db-collections/{collection_id}",
    request_body = crate::routes::user_db_collections::update_user_db_collection_request::UpdateUserDbCollectionRequest,
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the collection to update")
    ),
    responses(
        (status = 200, description = "User DB collection updated successfully", body = crate::db::user_db_collection::UserDbCollection),
        (status = 400, description = "Invalid request payload", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Collection not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::put("/{collection_id}")]
pub async fn update_user_db_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id: actix_web::web::Path<uuid::Uuid>,
    req_body: actix_web::web::Json<crate::routes::user_db_collections::update_user_db_collection_request::UpdateUserDbCollectionRequest>,
) -> actix_web::HttpResponse {
    let user_id_from_claims = claims.user_id;
    let collection_id_to_update = collection_id.into_inner();

    match crate::queries::user_db_collections::update_user_db_collection_query::update_user_db_collection_query(
        pool.get_ref(),
        user_id_from_claims,
        collection_id_to_update,
       req_body.name.clone(),
       req_body.description.clone(),
   ).await
   {
       Ok(collection) => actix_web::HttpResponse::Ok().json(collection),
       Err(sqlx::Error::RowNotFound) => actix_web::HttpResponse::NotFound().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Collection not found or not owned by user."),
            },
        ),
        Err(e) => {
            log::error!("Failed to update user DB collection: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to update collection."),
                },
            )
        }
    }
}
