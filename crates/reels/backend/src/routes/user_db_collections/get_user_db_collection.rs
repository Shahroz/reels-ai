//! Handler for retrieving a specific user DB collection by its ID.
//!
//! This endpoint allows an authenticated user to fetch details of a single
//! custom database collection, provided it belongs to them.
//! Adheres to 'one item per file' and FQN guidelines.

/// Retrieves a specific DB collection by ID.
///
/// Fetches the details of a single custom database collection if it exists
/// and belongs to the authenticated user.
#[utoipa::path(
    get,
    path = "/api/user-db-collections/{collection_id}",
    params(
        ("collection_id" = uuid::Uuid, Path, description = "ID of the collection to retrieve")
    ),
    responses(
        (status = 200, description = "User DB collection details", body = crate::db::user_db_collection::UserDbCollection),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Collection not found or not owned by user", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User DB Collections"
)]
#[actix_web::get("/{collection_id}")]
pub async fn get_user_db_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    collection_id: actix_web::web::Path<uuid::Uuid>,
) -> actix_web::HttpResponse {
    let user_id_from_claims = claims.user_id;
    let collection_id_to_fetch = collection_id.into_inner();

    match crate::queries::user_db_collections::get_user_db_collection_query::get_user_db_collection_query(
        pool.get_ref(),
        user_id_from_claims,
        collection_id_to_fetch,
    )
    .await
    {
        Ok(Some(collection)) => actix_web::HttpResponse::Ok().json(collection),
        Ok(None) => actix_web::HttpResponse::NotFound().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Collection not found or not owned by user."),
            },
        ),
        Err(e) => {
            log::error!("Failed to get user DB collection: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve collection."),
                },
            )
        }
    }
}
