//! Defines the `get_asset_by_id` HTTP route handler.
//!
//! This handler retrieves a single asset by its ID, including collection details
//! if the asset belongs to a collection.
//! Adheres to the project's Rust coding standards.



/// Get asset by ID with collection details
///
/// Retrieves a single asset by its ID. If the asset belongs to a collection,
/// the collection details will be included in the response.
#[utoipa::path(
    get,
    path = "/api/assets/{id}",
    tag = "Assets",
    params(
        ("id" = String, Path, description = "Asset ID")
    ),
    responses(
        (status = 200, description = "Asset found", body = crate::routes::assets::responses::AssetWithCollection),
        (status = 404, description = "Asset not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::get("{id}")]
#[tracing::instrument(skip(pool, claims))]
pub async fn get_asset_by_id(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let asset_id = path.into_inner();
    let user_id = claims.user_id;

    match crate::queries::assets::get_asset_by_id_with_collection::get_asset_by_id_with_collection(&pool, asset_id, user_id).await {
        Ok(Some(asset)) => {
            // The query already handles all permission checks (ownership, public access, and organization shares)
            // No additional permission checking needed here
            actix_web::HttpResponse::Ok().json(asset)
        }
        Ok(None) => actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
            error: "Asset not found".to_string(),
        }),
        Err(e) => {
            log::error!("Database error in get_asset_by_id: {e}");
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Internal server error".to_string(),
            })
        }
    }
}
