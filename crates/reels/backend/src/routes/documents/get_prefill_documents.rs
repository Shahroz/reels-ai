// Defines the GET `/api/documents/prefill-documents` endpoint.
// This endpoint retrieves documents for the authenticated user that are marked
// to always be included, typically used for pre-filling context
// or providing standard information for documents. It relies on the `fetch_always_include_documents_for_user`
// database function.
// Adheres to Rust coding guidelines, including one item per file and comprehensive documentation.

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

use crate::auth::tokens::Claims;
use crate::db::documents::Document;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    get,
    path = "/api/documents/prefill-documents",
    tag = "Documents",
    responses(
        (status = 200, description = "List of documents marked for always including, suitable for prefill.", body = Vec<Document>),
        (status = 401, description = "Unauthorized - JWT token missing or invalid."),
        (status = 500, description = "Internal Server Error - Failed to fetch documents.", body = ErrorResponse)
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("/prefill-documents")] // Path relative to the /api/documents scope
#[instrument(skip(pool, claims))]
pub async fn get_prefill_documents(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;

    match crate::queries::documents::fetch_always_include_documents::fetch_always_include_documents_for_user(&pool, user_id).await {
        Ok(documents) => {
            log::info!("Successfully fetched {} prefill documents for user {}", documents.len(), user_id);
            HttpResponse::Ok().json(documents)
        }
        Err(e) => {
            log::error!("Failed to fetch prefill documents for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve prefill documents.".to_string(),
            })
        }
    }
}
