//! Deletes a document entry by ID for the authenticated user.
//!
//! This handler processes DELETE requests to `/api/documents/{id}`,
//! removing the specified document task if it belongs to the user.
//! Returns 204 No Content on success, 404 if not found, or 500 on error.

#[utoipa::path(
    delete,
    path = "/api/documents/{id}",
    tag = "Documents",
    params(
        ("id" = String, Path, description = "Document entry ID")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::delete("/{id}")]
pub async fn delete_document(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let document_id = path.into_inner();
    let result = crate::queries::documents::delete_document_entry::delete_document_entry(
        &pool,
        document_id,
        user_id,
    )
    .await;

    match result {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                actix_web::HttpResponse::NoContent().finish()
            } else { // rows_affected == 0
                actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Document not found"),
                })
            }
        }
        Err(e) => {
            log::error!(
                "Error deleting document {document_id} for user {user_id}: {e}"
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Failed to delete document"),
            })
        }
    }
}
