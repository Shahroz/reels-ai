//! Handler for retrieving a vocal tour by its document ID.
//!
//! Defines the `get_vocal_tour_by_document` HTTP handler under `/api/vocal-tour/document/{document_id}`.
//! This handler checks if a document has an associated vocal tour and returns it if found.
//! Includes proper authorization to ensure users can only access their own vocal tours.



#[utoipa::path(
    get,
    path = "/api/vocal-tour/document/{document_id}",
    tag = "Vocal Tour",
    params(
        ("document_id" = uuid::Uuid, Path, description = "Document ID to find vocal tour for")
    ),
    responses(
        (status = 200, description = "Vocal tour found", body = crate::db::vocal_tours::VocalTour),
        (status = 404, description = "Vocal tour not found for this document"),
        (status = 403, description = "User does not own the document"), 
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::get("/document/{document_id}")]
#[tracing::instrument(skip(pool, claims))]
pub async fn get_vocal_tour_by_document(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<uuid::Uuid>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let document_id = path.into_inner();
    let user_id = claims.user_id;

    // First verify that the user owns the document
    match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
        &pool,
        document_id,
        user_id,
    ).await {
        std::result::Result::Ok(std::option::Option::Some(_)) => {
            // User owns the document, proceed to find vocal tour
        }
        std::result::Result::Ok(std::option::Option::None) => {
            return actix_web::HttpResponse::Forbidden().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Document not found or access denied".into(),
                }
            );
        }
        std::result::Result::Err(e) => {
            log::error!("Database error checking document ownership for document {document_id} and user {user_id}: {e}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Failed to verify document access".into(),
                }
            );
        }
    }

    // Find the vocal tour for this document
    match crate::queries::vocal_tours::get_vocal_tour_by_document_id::get_vocal_tour_by_document_id(
        &pool,
        document_id,
    ).await {
        std::result::Result::Ok(std::option::Option::Some(vocal_tour)) => {
            actix_web::HttpResponse::Ok().json(vocal_tour)
        }
        std::result::Result::Ok(std::option::Option::None) => {
            actix_web::HttpResponse::NotFound().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "No vocal tour found for this document".into(),
                }
            )
        }
        std::result::Result::Err(e) => {
            log::error!("Database error fetching vocal tour for document {document_id}: {e}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Failed to fetch vocal tour".into(),
                }
            )
        }
    }
} 