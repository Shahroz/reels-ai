//! Handler for deleting a logo collection.
//!
//! DELETE /api/logo-collections/{id}

use actix_web::{delete, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::routes::error_response::ErrorResponse;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteLogoCollectionResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/api/logo-collections/{id}",
    tag = "Logo Collections",
    params(
        ("id" = Uuid, Path, description = "Logo collection ID")
    ),
    responses(
        (status = 200, description = "Success", body = DeleteLogoCollectionResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[delete("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_logo_collection(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;

    let result = crate::queries::logo_collections::delete_logo_collection::delete_logo_collection(
        pool.get_ref(),
        collection_id,
        user_id,
    )
    .await;

    match result {
        Ok(true) => HttpResponse::Ok().json(DeleteLogoCollectionResponse {
            success: true,
            message: "Logo collection deleted successfully".to_string(),
        }),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Logo collection not found".to_string(),
        }),
        Err(err) => {
            eprintln!("Database error deleting logo collection: {err:?}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete logo collection".to_string(),
            })
        }
    }
}
